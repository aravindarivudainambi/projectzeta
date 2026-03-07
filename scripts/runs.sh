#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
LOG_DIR="$ROOT_DIR/logs/runs"

SERVICE_NAMES=()
SERVICE_PIDS=()
SERVICE_LOGS=()

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
  exit "$exit_code"
}

trap cleanup EXIT INT TERM

# Free a TCP port by killing any process currently bound to it.
free_port() {
  local port="$1"
  local pids
  pids="$(lsof -ti :"$port" 2>/dev/null || true)"
  if [[ -n "$pids" ]]; then
    echo "Freeing port $port (pid $pids)..."
    echo "$pids" | xargs kill -9 2>/dev/null || true
    sleep 0.5
  fi
}

require_cmd cargo
require_cmd curl
require_cmd pnpm

mkdir -p "$LOG_DIR"
cd "$ROOT_DIR"

if [[ ! -d "$ROOT_DIR/node_modules" ]]; then
  echo "Installing JavaScript dependencies..."
  pnpm install
fi

# Kill any stale processes from a previous run before binding ports.
free_port 8080
free_port 3000

start_service "api-gateway" cargo run -p api-gateway
start_service "web" pnpm --filter web dev

wait_for_http "http://127.0.0.1:8080/health" "api-gateway health endpoint"
wait_for_http "http://127.0.0.1:3000" "web frontend"

echo ""
echo "Both services are running."
echo "  Web:     http://localhost:3000"
echo "  Gateway: http://localhost:8080/health"
echo "  Logs:    $LOG_DIR"
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
