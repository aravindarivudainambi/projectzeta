.PHONY: dev run build test lint gen-types migrate

# Start all services with Docker infra (api-gateway, agent-engine, connector-hub,
# auth-service, observability-service, web). Requires Docker for postgres/redis/minio.
dev:
	@./scripts/dev.sh

# Start only the three core services needed for local iteration without Docker:
# api-gateway (8080), connector-hub (8082), web (3000).
run:
	@./scripts/runs.sh

# Build the full Rust workspace and type-check the Next.js app.
build:
	cargo build --workspace
	pnpm --filter web exec tsc --noEmit

# Run Rust unit tests across the workspace and the web app's test script.
test:
	cargo test --workspace
	pnpm --filter web run test

# Check formatting and run Clippy across the Rust workspace; run Next.js ESLint.
lint:
	cargo fmt --all -- --check
	cargo clippy --workspace
	pnpm --filter web run lint

# Regenerate packages/schema-types/src/index.ts from Rust core-types definitions.
gen-types:
	@./scripts/gen-types.sh

# Placeholder: run SQLx migrations once the database layer is wired up.
migrate:
	@echo "Run SQLx migrations once service wiring is implemented."
