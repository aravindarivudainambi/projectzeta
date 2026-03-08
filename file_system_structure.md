
# Internal Agent Builder — File System Structure

## Monorepo Root

```
agent-builder/
├── Cargo.toml                     # Cargo workspace — defines all Rust crates
├── Cargo.lock
├── pnpm-workspace.yaml            # pnpm workspace — defines all JS packages
├── package.json                   # Root scripts (dev, build, lint, test)
├── .env.example
├── docker-compose.yml             # Local dev: Postgres, Redis, Vault, MinIO
├── Makefile                       # Shortcuts: make dev, make migrate, make test
├── .github/
│   └── workflows/
│       ├── ci.yml                 # Test + lint on PR
│       └── deploy.yml             # Build + push Docker images on merge
│
├── apps/                          # Deployable applications
│   ├── api-gateway/               # Rust — Axum gateway service
│   ├── agent-engine/              # Rust — agent orchestration service
│   ├── connector-hub/             # Rust — MCP client + integrations
│   ├── auth-service/              # Rust — IAM, JWT, RBAC
│   ├── observability-service/     # Rust — telemetry, cost tracking
│   └── web/                       # Next.js 15 — frontend
│
├── libs/                          # Shared Rust libraries (workspace crates)
│   ├── core-types/                # Shared domain types (AgentConfig, ToolSchema, etc.)
│   ├── db/                        # Shared Postgres pool, migrations, RLS helpers
│   ├── secret-vault/              # Credential storage abstraction
│   ├── llm-client/                # Unified LLM provider client (OpenAI, Anthropic, etc.)
│   ├── mcp-sdk/                   # MCP protocol implementation
│   └── telemetry/                 # Shared tracing/metrics setup
│
├── packages/                      # Shared JS/TS packages
│   ├── ui/                        # shadcn/ui component library
│   ├── schema-types/              # Auto-generated TypeScript types from Rust JSON Schemas
│   └── api-client/                # Type-safe fetch client for all backend endpoints
│
├── scripts/
│   ├── gen-types.sh               # Runs schemars → json-schema-to-typescript
│   └── seed-db.sh                 # Seed local Postgres with test data
│
├── migrations/                    # sqlx migration files (run by api-gateway at startup)
│   ├── 0001_initial_schema.sql
│   ├── 0002_add_agent_versions.sql
│   └── 0003_add_rls_policies.sql
│
└── docs/
    ├── architecture.md
    └── adr/                       # Architecture Decision Records
        ├── 001-rust-axum.md
        └── 002-postgres-rls-multitenancy.md
```

---

## Rust Services (apps/)

All services follow the same internal layout:

```
apps/api-gateway/
├── Cargo.toml
└── src/
    ├── main.rs                    # Starts Tokio runtime, binds port
    ├── app.rs                     # Builds Axum router, attaches all middleware
    ├── config.rs                  # Loads .env via envy, typed config struct
    │
    ├── middleware/
    │   ├── mod.rs
    │   ├── auth.rs                # JWT extraction → inject UserId into request extensions
    │   ├── tenant.rs              # Sets app.current_tenant_id on DB connection
    │   └── rate_limit.rs          # governor token bucket per user
    │
    ├── routes/
    │   ├── mod.rs
    │   ├── agents.rs              # POST /agents, GET /agents/:id, PATCH, DELETE
    │   ├── runs.rs                # POST /runs, GET /runs/:id, SSE /runs/:id/stream
    │   ├── connectors.rs          # GET /connectors, POST /connectors/connect
    │   ├── marketplace.rs         # GET /marketplace, POST /marketplace/fork
    │   └── health.rs              # GET /health, GET /ready
    │
    └── errors.rs                  # AppError enum → HTTP response mapping
```

```
apps/agent-engine/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── config.rs
    │
    ├── executor/
    │   ├── mod.rs
    │   ├── runner.rs              # Core agent loop: plan → tool calls → emit events
    │   ├── planner.rs             # LLM call to generate next step
    │   ├── tool_caller.rs         # Dispatches tool invocations via connector-hub
    │   └── event_emitter.rs      # Sends AgentEvent variants down SSE channel
    │
    ├── human_loop/
    │   ├── mod.rs
    │   └── approvals.rs           # Pauses run, writes ApprovalRequest to DB, waits
    │
    ├── memory/
    │   ├── mod.rs
    │   └── context.rs             # Builds prompt context window from prior steps + RAG
    │
    ├── versioning/
    │   ├── mod.rs
    │   ├── snapshot.rs            # Captures full behavioral bundle into agent_versions
    │   └── rollback.rs            # Restores an agent_version as the active config
    │
    └── events.rs                  # AgentEvent enum (StepStarted, ToolCalled, HumanApproval, Finished)
```

```
apps/connector-hub/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── config.rs
    │
    ├── registry/
    │   ├── mod.rs
    │   └── connectors.rs          # Catalog of available integrations + their MCP server URLs
    │
    ├── mcp/
    │   ├── mod.rs
    │   ├── client.rs              # MCP protocol client (tool discovery + invocation)
    │   └── transport.rs           # stdio and SSE MCP transport implementations
    │
    ├── oauth/
    │   ├── mod.rs
    │   ├── flow.rs                # PKCE authorization code flow
    │   └── refresh.rs             # Background token refresh loop
    │
    ├── vault/
    │   ├── mod.rs
    │   └── store.rs               # Encrypted credential storage + JIT lookup
    │
    └── adapters/                  # Per-tool adapters where MCP isn't available yet
        ├── notion.rs
        ├── google_workspace.rs
        └── mod.rs
```

```
apps/auth-service/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── config.rs
    │
    ├── tokens/
    │   ├── mod.rs
    │   ├── issue.rs               # Signs JWT for users + separate machine tokens for agents
    │   └── validate.rs            # Validates + decodes tokens, checks revocation list
    │
    ├── rbac/
    │   ├── mod.rs
    │   ├── policies.rs            # casbin-rs policy definitions
    │   └── enforcer.rs            # "Can agent X invoke tool Y on behalf of user Z?"
    │
    └── users.rs                   # User CRUD, password hashing with argon2
```

```
apps/observability-service/
├── Cargo.toml
└── src/
    ├── main.rs
    ├── config.rs
    │
    ├── collector/
    │   ├── mod.rs
    │   └── ingest.rs              # Receives AgentRunComplete events, writes to DB
    │
    ├── cost/
    │   ├── mod.rs
    │   └── calculator.rs          # tokens * price_per_token → cost_usd, per model
    │
    └── aggregates/
        ├── mod.rs
        └── queries.rs             # Pre-aggregated SQL for dashboard charts
```

---

## Shared Rust Libraries (libs/)

```
libs/core-types/
└── src/
    ├── lib.rs
    ├── agent.rs                   # AgentConfig, AgentVersion, AgentStatus
    ├── tool.rs                    # ToolSchema, ToolCall, ToolResult
    ├── run.rs                     # AgentRun, RunStep, RunStatus
    ├── user.rs                    # User, Tenant, Permission
    └── events.rs                  # AgentEvent (SSE payload types)
```

```
libs/db/
└── src/
    ├── lib.rs
    ├── pool.rs                    # PgPool setup with deadpool-postgres
    ├── rls.rs                     # Sets current_tenant_id on connection checkout
    └── macros.rs                  # sqlx query! wrappers with RLS context
```

```
libs/llm-client/
└── src/
    ├── lib.rs
    ├── provider.rs                # LlmProvider trait
    ├── router.rs                  # Model router: cost vs. latency vs. capability
    ├── openai.rs
    ├── anthropic.rs
    └── pii_scrubber.rs            # Regex + NER-based PII redaction before API calls
```

---

## Frontend (apps/web/)

```
apps/web/
├── package.json
├── next.config.ts
├── tailwind.config.ts
├── tsconfig.json
│
├── app/                           # Next.js App Router — all routes live here
│   ├── layout.tsx                 # Root layout: auth provider, theme, fonts
│   ├── page.tsx                   # Landing / redirect to /dashboard
│   │
│   ├── (auth)/                    # Route group — no sidebar layout
│   │   ├── login/page.tsx
│   │   └── callback/page.tsx      # OAuth callback handler
│   │
│   ├── (app)/                     # Route group — authenticated shell with sidebar
│   │   ├── layout.tsx             # Sidebar + top nav wrapper
│   │   │
│   │   ├── dashboard/
│   │   │   └── page.tsx           # RSC: agent list, recent runs, usage summary
│   │   │
│   │   ├── agents/
│   │   │   ├── page.tsx           # RSC: all agents list
│   │   │   ├── new/
│   │   │   │   └── page.tsx       # Client: NL builder + visual canvas
│   │   │   └── [agentId]/
│   │   │       ├── page.tsx       # RSC: agent detail + run history
│   │   │       ├── edit/page.tsx  # Client: edit agent config
│   │   │       └── versions/
│   │   │           └── page.tsx   # RSC: behavioral version history + rollback UI
│   │   │
│   │   ├── runs/
│   │   │   └── [runId]/
│   │   │       └── page.tsx       # Client: live run viewer with SSE stream
│   │   │
│   │   ├── marketplace/
│   │   │   ├── page.tsx           # RSC: template catalog
│   │   │   └── [templateId]/
│   │   │       └── page.tsx       # RSC: template detail + fork button
│   │   │
│   │   ├── connectors/
│   │   │   └── page.tsx           # Client: connect SaaS tools, OAuth flows
│   │   │
│   │   ├── observability/
│   │   │   ├── page.tsx           # RSC: cost dashboard + usage charts
│   │   │   └── [agentId]/
│   │   │       └── page.tsx       # RSC: per-agent telemetry deep dive
│   │   │
│   │   └── settings/
│   │       ├── page.tsx           # Team settings, billing, permissions
│   │       └── members/page.tsx
│   │
│   └── api/                       # Next.js Route Handlers (thin proxy to Rust)
│       ├── agent/
│       │   └── build/route.ts     # Streams NL → agent config via Rust SSE
│       └── auth/
│           └── [...nextauth]/route.ts
│
├── components/
│   ├── agent-builder/
│   │   ├── NLBuilder.tsx          # useChat + streaming token display
│   │   ├── WorkflowCanvas.tsx     # reactflow drag-and-drop editor
│   │   └── AgentPreview.tsx       # Live preview of agent config being built
│   │
│   ├── run-viewer/
│   │   ├── LiveRunViewer.tsx      # EventSource subscriber, step tree renderer
│   │   ├── StepCard.tsx           # Tool call card: name, args, result, latency
│   │   └── ApprovalModal.tsx      # Interrupts UI for human-in-the-loop sign-off
│   │
│   ├── observability/
│   │   ├── CostChart.tsx          # Recharts cost-over-time per agent
│   │   └── LatencyHeatmap.tsx
│   │
│   └── ui/                        # shadcn/ui components (Button, Card, Dialog, etc.)
│
├── hooks/
│   ├── useAgentRun.ts             # Subscribes to SSE run stream
│   ├── useAgentBuilder.ts         # Wraps useChat for NL builder UX
│   └── useApproval.ts             # Manages approval modal state
│
├── lib/
│   ├── api.ts                     # Typed fetch client (wraps @packages/api-client)
│   ├── auth.ts                    # NextAuth config
│   └── sse.ts                     # EventSource wrapper with reconnect logic
│
└── public/
    └── icons/
```

---

## Shared JS Packages (packages/)

```
packages/schema-types/
├── package.json
└── src/
    └── index.ts                   # Auto-generated from Rust schemars output
                                   # Run: pnpm gen-types to regenerate

packages/api-client/
├── package.json
└── src/
    ├── index.ts
    ├── agents.ts                  # createAgent(), getAgent(), listAgents()
    ├── runs.ts                    # startRun(), streamRun()
    └── connectors.ts              # listConnectors(), connectOAuth()

packages/ui/
├── package.json
└── src/
    └── components/                # Shared design system components
```
