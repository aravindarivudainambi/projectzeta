#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/logs/dev"

SERVICE_NAMES=()
SERVICE_PIDS=()
SERVICE_LOGS=()
STARTED_INFRA=0

require_cmd() {
  local cmd="$1"
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo "Missing required command: $cmd"
    exit 1
  fi
}

wait_for_http() {
  local url="$1"
  local label="$2"
  local timeout_seconds="${3:-120}"
  local started_at
  started_at="$(date +%s)"

  while true; do
    if curl --silent --fail "$url" >/dev/null 2>&1; then
      return 0
    fi

    if (( "$(date +%s)" - started_at >= timeout_seconds )); then
      echo "Timed out waiting for $label at $url"
      return 1
    fi
    sleep 1
  done
}

start_service() {
  local name="$1"
  shift
  local log_file="$LOG_DIR/$name.log"

  echo "Starting $name..."
  "$@" >"$log_file" 2>&1 &
  local pid="$!"

  SERVICE_NAMES+=("$name")
  SERVICE_PIDS+=("$pid")
  SERVICE_LOGS+=("$log_file")
}

cleanup() {
  local exit_code="$?"
  trap - EXIT INT TERM

  for pid in "${SERVICE_PIDS[@]:-}"; do
    if kill -0 "$pid" >/dev/null 2>&1; then
      kill "$pid" >/dev/null 2>&1 || true
    fi
  done
  wait >/dev/null 2>&1 || true

  if [[ "$STARTED_INFRA" -eq 1 ]]; then
    docker compose down >/dev/null 2>&1 || true
  fi
  exit "$exit_code"
}

trap cleanup EXIT INT TERM

require_cmd cargo
require_cmd curl
require_cmd pnpm

mkdir -p "$LOG_DIR"
cd "$ROOT_DIR"

if [[ ! -d "$ROOT_DIR/node_modules" ]]; then
  echo "Installing JavaScript dependencies..."
  pnpm install
fi

if command -v docker >/dev/null 2>&1; then
  echo "Booting shared infrastructure containers..."
  docker compose up -d postgres redis minio
  STARTED_INFRA=1
else
  echo "Docker not found; skipping postgres/redis/minio containers."
fi

start_service "api-gateway" cargo run -p api-gateway
start_service "agent-engine" cargo run -p agent-engine
start_service "connector-hub" cargo run -p connector-hub
start_service "auth-service" cargo run -p auth-service
start_service "observability-service" cargo run -p observability-service
start_service "web" pnpm --filter web dev

wait_for_http "http://127.0.0.1:8080/health" "api-gateway health endpoint"

api_gateway_health_body="$(curl --silent --fail http://127.0.0.1:8080/health)"
if [[ "$api_gateway_health_body" != '{"status":"ok"}' ]]; then
  echo "Unexpected api-gateway health body: $api_gateway_health_body"
  exit 1
fi

wait_for_http "http://127.0.0.1:8081/health" "agent-engine health endpoint"
wait_for_http "http://127.0.0.1:8082/health" "connector-hub health endpoint"
wait_for_http "http://127.0.0.1:8083/health" "auth-service health endpoint"
wait_for_http "http://127.0.0.1:8084/health" "observability-service health endpoint"
wait_for_http "http://127.0.0.1:3000" "web home page"

echo "Development stack is running."
echo "Web: http://localhost:3000"
echo "Gateway health: http://localhost:8080/health"
echo "Logs: $LOG_DIR"
echo "Press Ctrl+C to stop."

while true; do
  for i in "${!SERVICE_PIDS[@]}"; do
    if ! kill -0 "${SERVICE_PIDS[$i]}" >/dev/null 2>&1; then
      echo "Service '${SERVICE_NAMES[$i]}' exited unexpectedly."
      echo "Last log lines from ${SERVICE_LOGS[$i]}:"
      tail -n 40 "${SERVICE_LOGS[$i]}" || true
      exit 1
    fi
  done
  sleep 2
done
