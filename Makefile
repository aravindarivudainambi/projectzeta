.PHONY: dev migrate test lint

dev:
	@./scripts/dev.sh

migrate:
	@echo "Run SQLx migrations once service wiring is implemented."

test:
	@echo "Add Rust and TypeScript test runners once implementations exist."

lint:
	@echo "Add cargo fmt/clippy and pnpm lint once implementations exist."
