
# Internal Agent Builder — Full-Stack Architecture
## Stack: Rust (Axum/Tokio) Backend + Next.js 15 App Router Frontend

---

## High-Level System Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                      FRONTEND (Next.js 15)                      │
│  App Router │ React Server Components │ Vercel AI SDK │ Tailwind │
│                                                                   │
│  ┌──────────┐  ┌──────────────┐  ┌──────────────┐  ┌─────────┐ │
│  │  Agent   │  │   Workflow   │  │ Marketplace  │  │ Monitor │ │
│  │ Builder  │  │    Canvas    │  │  & Templates │  │ & Logs  │ │
│  │  (NL UI) │  │  (Drag+Drop) │  │              │  │         │ │
│  └──────────┘  └──────────────┘  └──────────────┘  └─────────┘ │
└───────────────────────────┬─────────────────────────────────────┘
                            │  HTTPS + SSE / WebSocket
┌───────────────────────────▼─────────────────────────────────────┐
│                  API GATEWAY (Rust / Axum)                       │
│         Auth Middleware │ Rate Limiting │ Request Tracing         │
└───┬──────────────┬──────────────┬───────────────┬───────────────┘
    │              │              │               │
    ▼              ▼              ▼               ▼
┌────────┐  ┌──────────┐  ┌──────────┐  ┌────────────────┐
│ Agent  │  │Connector │  │  Auth /  │  │  Observability │
│Engine  │  │  Hub     │  │  IAM     │  │  Service       │
│Service │  │  (MCP)   │  │ Service  │  │  (Traces/Costs)│
└───┬────┘  └────┬─────┘  └────┬─────┘  └───────┬────────┘
    │            │             │                 │
    ▼            ▼             ▼                 ▼
┌──────────────────────────────────────────────────────────┐
│                    DATA LAYER                            │
│  PostgreSQL (RLS)  │  Redis  │  S3/Object Store          │
│  (agents, perms,   │  (cache,│  (agent artifacts,        │
│   audit logs, RAG  │  queues)│   prompt snapshots)        │
│   vector store)    │         │                            │
└──────────────────────────────────────────────────────────┘
    │
    ▼
┌──────────────────────────────────────────────────────────┐
│                  EXTERNAL INTEGRATIONS                   │
│  LLM Providers │ SaaS Tools (via MCP) │ Internal APIs    │
│  (OpenAI, Anthropic, │  (Google Workspace,│  (via OAuth +  │
│   Gemini, local)     │   Notion)          │   token vault) │
└──────────────────────────────────────────────────────────┘
```

---

## Backend Services (Rust)

### 1. API Gateway  (`axum` + `tower` middleware)
The single ingress point. Every request passes through this layer.

**Libraries:**
- `axum` — type-safe, async HTTP routing
- `tower` — composable middleware (auth, rate limiting, timeouts, compression)
- `tower-http` — CORS, tracing, request IDs
- `jsonwebtoken` — JWT validation for user and agent tokens
- `governor` — token-bucket rate limiting per user/tenant

**Key Middleware Stack (applied in order):**
```
TraceLayer → RequestIdLayer → AuthLayer → TenantLayer → RateLimitLayer → Handler
```

**Why Rust here:** Axum handles ~500k req/s on a single core with ~2MB RSS vs ~200MB for Node. Every millisecond saved here multiplies across every agent run.

---

### 2. Agent Engine Service  (`tokio` + async channels)
The core orchestration loop. Runs agent step-by-step, manages tool calls, handles retries and timeouts.

**Libraries:**
- `tokio` — async runtime, spawning agent tasks as lightweight green threads
- `tokio-stream` — back-pressured streaming of agent events to frontend
- `serde` / `serde_json` — typed tool schemas and LLM I/O serialization
- `reqwest` — async HTTP client for LLM API calls
- `async-channel` — fan-out agent events to SSE subscribers

**Agent Execution Loop:**
```
receive_task()
  → load_agent_config(agent_id)
  → plan_steps(llm_call)  ← streams partial tokens to frontend via SSE
  → for each step:
      → check_permissions(action, user_context)
      → if sensitive_action: pause + emit HumanApprovalEvent
      → else: invoke_tool(mcp_client, tool, args)
      → emit StepCompleteEvent
  → emit AgentFinishedEvent
  → write_audit_log()
```

**Streaming:** SSE endpoint in Axum using `axum::response::Sse` with an `mpsc` channel. Each token from the LLM and each tool result gets pushed downstream as a typed event. Frontend uses `useChat` from Vercel AI SDK to consume this.

---

### 3. Connector Hub  (MCP client + integration adapters)
Manages all outbound connections to SaaS tools and internal APIs.

**Libraries:**
- `mcp-client` (Rust MCP SDK) — model context protocol server discovery and tool invocation
- `oauth2` crate — PKCE + token refresh flows for every connected app
- `deadpool` — async connection pooling for database and external services
- `secrecy` — zero-cost secret type that prevents accidental logging of credentials

**Token Vault Pattern:**
```rust
// Dynamic, just-in-time credential lookup — never stored in agent config
async fn get_tool_credentials(
    vault: &SecretVault, 
    user_id: Uuid, 
    tool: &str
) -> Result<ScopedToken, VaultError> {
    vault.get_token(user_id, tool, TokenScope::ReadWrite).await
}
```

**Why this is better than static env vars:** Tokens are scoped per user, per tool, per run. Revocation is instant. No credential sprawl.

---

### 4. Auth / IAM Service
Handles identity for both *humans* (who build agents) and *agents themselves* (non-human principals).

**Libraries:**
- `argon2` — password hashing
- `jsonwebtoken` — short-lived JWT for users; agent tokens with embedded permission claims
- `casbin-rs` — RBAC/ABAC policy engine for evaluating "can agent X use tool Y on behalf of user Z?"

**Key concept — Agent Identity:**
Every agent gets a unique UUID-based identity. When the agent calls a tool, it presents:
- Its own agent token (what it is)
- The delegating user's context (on whose behalf)
- The requested scope (read-only vs. write)

PostgreSQL Row-Level Security (RLS) enforces tenant isolation at the data layer so even a buggy query can't leak cross-tenant data.

---

### 5. Observability Service
Structured telemetry for every agent run — latency, token cost, tool outcomes.

**Libraries:**
- `tracing` + `tracing-subscriber` — structured, async-safe span logging
- `opentelemetry` (Rust SDK) — OTLP export to Grafana/Jaeger/Datadog
- `metrics` crate — Prometheus counters for token usage, latency, error rates

**What gets tracked per agent run:**
```
agent_run_id, user_id, tenant_id, agent_version,
steps: [{tool, latency_ms, tokens_in, tokens_out, success}],
total_cost_usd, human_approvals_requested, human_approvals_granted,
final_status
```

Cost attribution is calculated at write time: `tokens * (price_per_token for model)` — stored in Postgres, aggregated per team per billing period.

---

## Data Layer

| Store | Technology | Purpose |
|-------|-----------|---------|
| Primary DB | PostgreSQL 16 + pgvector | Agents, runs, users, audit logs, RAG vector store |
| Cache / Queues | Redis (Valkey) | Session cache, agent task queues, pub/sub for live events |
| Secret Storage | Encrypted Postgres + Vault | OAuth tokens, API keys, per-user credentials |
| Object Storage | S3-compatible (R2/MinIO) | Prompt snapshots, behavioral version bundles, file attachments |
| Search | PostgreSQL FTS / Meilisearch | Agent marketplace search, log search |

**Multi-tenancy:** Single-schema Postgres with RLS. Every table has a `tenant_id` column. A middleware sets `app.current_tenant_id` on every connection, and RLS policies filter automatically. Zero risk of cross-tenant data leakage even with raw SQL.

**Behavioral Versioning Schema:**
```sql
CREATE TABLE agent_versions (
  id           UUID PRIMARY KEY,
  agent_id     UUID NOT NULL,
  version      INT NOT NULL,
  snapshot     JSONB NOT NULL,  -- full bundle: prompts + tools + model + memory
  created_at   TIMESTAMPTZ,
  created_by   UUID,
  is_active    BOOLEAN DEFAULT false,
  metrics      JSONB  -- shadow-run comparison results
);
```

---

## Frontend (Next.js 15 App Router)

### Architecture Principles
- **React Server Components (RSC)** for all read-heavy views (agent list, run history, marketplace). Zero client JS bundle for these views.
- **Client Components** only where interactivity is needed: the workflow canvas, live agent run viewer, approval modals.
- **Streaming** via Next.js `loading.tsx` + Suspense boundaries for progressive rendering.

### Key UI Modules

**1. Agent Builder (Natural Language Mode)**
```tsx
// Vercel AI SDK useChat — streams tokens from Rust SSE endpoint
const { messages, input, handleSubmit, isLoading } = useChat({
  api: '/api/agent/build',
  onFinish: (msg) => triggerAgentPreview(msg.toolInvocations)
});
```
User types "Every Monday morning, summarize unread Gmail updates and save them to a Notion page." The backend LLM interprets this, generates a structured agent config, and streams it back. User sees a live preview build up token by token.

**2. Workflow Canvas (Visual Mode)**
A `reactflow`-based drag-and-drop canvas for users who prefer visual wiring. Nodes represent: Trigger → Condition → Tool Call → Human Review → Output. Serializes to the same JSON schema as the NL builder.

**3. Live Agent Run Viewer**
Uses `EventSource` (SSE) to subscribe to a specific `agent_run_id`. Renders a real-time step tree — each tool call appears as it executes, with latency badges, token counts, and expandable I/O panels. When a `HumanApprovalEvent` arrives, a modal interrupts the UI for sign-off.

**4. Observability Dashboard (RSC + Recharts)**
Server-rendered charts of token cost per agent over time, error rates, P50/P95 latency, and budget vs. actual spend — all from Postgres aggregates. No separate analytics database needed at early scale.

### Frontend Libraries
| Library | Role |
|---------|------|
| `@ai-sdk/react` (Vercel AI SDK) | `useChat`, streaming, tool invocation UI |
| `reactflow` | Workflow canvas drag-and-drop |
| `@tanstack/react-query` | Data fetching, cache invalidation |
| `zod` | Schema validation shared with Rust (via JSON Schema codegen) |
| `tailwindcss` + `shadcn/ui` | Design system |
| `recharts` | Cost/observability charts |

---

## Cross-Cutting Concerns

### Schema Sharing (Rust ↔ Next.js)
Define schemas once in Rust as `serde` structs → generate JSON Schema with `schemars` → import into TypeScript via `json-schema-to-typescript`. This eliminates the entire class of type mismatch bugs between backend and frontend.

### CI/CD & Deployment
- **Rust services:** Compiled to single static binaries (~15MB). Containerized with `scratch` or `distroless` base image. Deployed via Kubernetes or fly.io.
- **Next.js:** Deployed to Vercel (Edge Runtime for API routes that don't need Rust).
- **Database migrations:** `sqlx` migrations embedded in the Rust binary, run at startup.

### Security Hardening
- All LLM requests pass through the Rust gateway — PII regex-scrubbed before leaving the org.
- TLS everywhere, including between internal services.
- Agent tool calls are sandboxed: each has a 30s timeout and memory cap enforced at the Tokio task level.
- Prompt injection detection middleware in the Agent Engine (heuristic + small classifier model).

---

## Why Rust Is Specifically Powerful Here

Rust's biggest advantage over Python/Node for this use case isn't raw speed — it's **deterministic resource control under load**. When 50 agents are running simultaneously, each looping through tools with concurrent LLM calls, Python's GIL creates contention and Node's event loop can stall. Rust with Tokio spawns each agent as an independent task with no shared runtime bottleneck. The borrow checker also makes it nearly impossible to accidentally share mutable agent state between concurrent runs — a class of bug that would be catastrophic in production.

A Rust-based AI orchestration platform benchmarks at **15,000+ requests per second** with **45ms average response time** on commodity hardware.
