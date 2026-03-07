.PHONY: dev run migrate test lint

dev:
	@./scripts/dev.sh

run:
	@./scripts/runs.sh

migrate:
	@echo "Run SQLx migrations once service wiring is implemented."

test:
	@echo "Add Rust and TypeScript test runners once implementations exist."

lint:
	@echo "Add cargo fmt/clippy and pnpm lint once implementations exist."
