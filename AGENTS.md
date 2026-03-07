# AGENTS.md

## Purpose

This repository contains a scaffold for an internal agent builder platform. The codebase is intentionally light on implementation and heavy on contracts, structure, and documentation so future contributors can extend it safely.

## Working Rules

1. Keep the Rust and TypeScript layers contract-first.
2. Prefer adding interfaces, schemas, and typed placeholders before business logic.
3. Document every public function with an extensive Rust doc comment or JSDoc block.
4. Preserve the monorepo structure described in the architecture documents.
5. Reuse shared types from `libs/core-types` and `packages/schema-types` instead of redefining payloads.
6. Route all external integrations through the connector hub boundary.
7. Treat every multi-tenant data path as tenant-scoped by default.

## Implementation Expectations

- Rust service files should expose base functions, types, and module boundaries with `todo!()` placeholders until the real implementation is approved.
- Frontend route files should remain thin and delegate behavior to components, hooks, and client libraries.
- Shared packages should define stable interfaces that both the backend and frontend can depend on.
- Avoid embedding secrets, credentials, or environment-specific values in source files.

## Suggested Build Order

1. Finalize shared schemas in `libs/core-types`.
2. Wire database and tenant context helpers in `libs/db`.
3. Stand up API gateway and auth-service request boundaries.
4. Implement agent-engine orchestration and connector-hub tool dispatch.
5. Connect the Next.js app to the typed API client and streaming endpoints.
6. Add migrations, tests, telemetry, and deployment automation.

## Definition of Done for Future Work

- Public APIs are documented.
- New modules include tests or a documented reason they are still scaffolding.
- Cross-service contracts are reflected in both Rust and TypeScript types.
- Tenant and permission boundaries are explicit.

## Environment / Secrets

- **Location**: The local environment file lives at the repository root named `.env` (this file is gitignored).
- **Purpose**: Place private API keys and service endpoints used for local development here (for example, `GITHUB_MODELS_API_KEY` and `GITHUB_MODELS_BASE_URL`).
- **Security note**: Never commit secrets. Keep `.env` out of source control; use the tracked `.env.example` as a template for required variables.
