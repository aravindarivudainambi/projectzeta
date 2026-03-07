
# Internal Agent Builder — UI Specification Sheet

---

## How to Read This Spec

Each screen section contains:
- **Purpose** — what this screen is for and who uses it
- **Must-Have Components** — non-negotiable elements; without these the screen fails its job
- **Success Criteria** — measurable conditions that define "done" for the UI
- **Interaction Details** — exactly how key components behave
- **Failure States** — what the UI must show when things go wrong

A screen is considered **UI-complete** when every Must-Have Component is present AND every Success Criterion is met.

---

## Screen 1: Natural Language Builder

**Purpose:** Let any employee — technical or not — describe a workflow in plain English and receive a working agent config. This is the highest-leverage screen in the product. If it feels magical, the whole product wins.

### Must-Have Components

| Component | Description | Why It's Non-Negotiable |
|-----------|-------------|------------------------|
| **Prompt input textarea** | Multiline, auto-resizing, placeholder text with example workflow | First thing the user sees — must feel inviting, not technical |
| **Streaming JSON preview panel** | Side-by-side panel that builds the agent config token-by-token as LLM responds | Users must see the agent being constructed live; a loading spinner is not acceptable |
| **Token cursor indicator** | Blinking cursor at the end of the streaming text, just like a terminal | Communicates "it's actively working" without a spinner |
| **Tool badge list** | After the config is complete, auto-render pill badges for each detected tool (e.g. "Slack", "GitHub") | Users must immediately see which integrations will be invoked |
| **Edit before saving panel** | Once streaming completes, all fields (name, trigger, steps) are directly editable inline | LLM output is imperfect — the user must be able to fix it before committing |
| **Save Agent button** | Disabled during streaming, enabled only when config is valid JSON | Must never save a partial or invalid config |
| **Validation error inline** | If saved config fails schema validation, highlight the exact field in red | Generic "something went wrong" toasts are not acceptable here |
| **Visual mode toggle** | Button to switch the streaming preview to the Workflow Canvas view | Users who prefer visual wiring should not be blocked |

### Success Criteria

- User types a description and sees the first token appear in the preview panel within **1.5 seconds**
- The preview panel renders partial JSON gracefully — no visual flicker or layout jumps as tokens arrive
- At least **3 example prompts** are shown as clickable chips below the input (e.g. "Post standup to Slack every Friday", "Summarize PRs weekly", "Alert me when Notion task changes")
- Clicking an example chip populates the textarea and auto-submits — zero friction path to first agent
- The Save button becomes active **only** after the streaming ends and the JSON passes schema validation
- If the LLM returns invalid JSON: show a red banner reading "Config couldn't be parsed — try rephrasing" with a Retry button, never a raw error stack trace
- Tab order: textarea → Submit → Visual mode toggle (keyboard-navigable without a mouse)

### Failure States

| Scenario | What UI Must Show |
|----------|------------------|
| LLM API timeout (>10s) | "Taking longer than usual…" message with a Cancel button after 5s |
| LLM returns invalid JSON | Red banner, Retry button, preserve the original prompt text |
| No tools detected in config | Yellow warning: "No integrations detected — is your workflow missing a tool?" |
| User submits empty textarea | Input border turns red, shake animation, no API call made |

---

## Screen 2: Workflow Canvas (Visual Builder)

**Purpose:** A drag-and-drop alternative to the NL builder for users who prefer visual workflow composition, or for editing complex multi-step agents after initial creation.

### Must-Have Components

| Component | Description | Why It's Non-Negotiable |
|-----------|-------------|------------------------|
| **Node types** | At minimum: TriggerNode, StepNode (tool call), ConditionNode (if/else), OutputNode, HumanApprovalNode | Without HumanApprovalNode, users cannot mark high-stakes steps for review |
| **Connection lines with arrows** | Directed edges between nodes showing execution flow | Users must instantly understand the sequence — undirected graphs are ambiguous |
| **Node detail sidebar** | Clicking a node opens a right panel to configure its parameters (tool name, args, timeout) | Every node must be configurable; non-editable nodes are unusable |
| **Add node palette** | Left-side drawer listing all available tools as draggable items | Users need discoverability — they cannot guess tool names from memory |
| **Minimap** | Small overview in bottom-right corner for large graphs | Agents with 10+ steps become unnavigable without a minimap |
| **Undo / Redo** | Cmd+Z / Cmd+Shift+Z support | Any canvas editor without undo will cause rage-quits |
| **Validate & Save button** | Validates the graph (no orphaned nodes, all required fields filled) before serializing to AgentConfig | Must never save a broken graph |
| **Sync indicator** | Shows "Synced" or "Unsaved changes" badge in the top bar | Users must always know if their work is persisted |

### Success Criteria

- A saved `AgentConfig` with 3 steps renders as the correct number of connected nodes within **500ms** of page load
- Dragging a tool from the palette onto the canvas creates a new StepNode with the tool pre-populated
- Connecting two nodes creates a directed edge; disconnecting removes it — both update the underlying config state immediately
- The canvas is pannable (click-drag on background) and zoomable (scroll wheel) from 25% to 200%
- Keyboard shortcut `Delete` on a selected node removes it and its connected edges after a 1-click confirmation
- The Validate button catches: orphaned nodes (no connections), required fields left empty, cycles in the graph (infinite loops)
- Undo/redo history persists at least **50 operations**

### Failure States

| Scenario | What UI Must Show |
|----------|------------------|
| Cycle detected in graph | Red glow on the cyclic edge, tooltip: "This creates an infinite loop" |
| Orphaned node at save | Orange glow on the disconnected node, "Connect or remove this node" |
| Node has missing required fields | Red dot badge on the node, sidebar auto-opens to the missing field |
| Canvas fails to render config | Fallback to NL builder view with error: "Couldn't render as canvas" |

---

## Screen 3: Live Run Viewer

**Purpose:** Show an agent executing in real time — every tool call, every decision, every token — so users can trust and debug what the agent is doing. This is the transparency engine of the product.

### Must-Have Components

| Component | Description | Why It's Non-Negotiable |
|-----------|-------------|------------------------|
| **Step tree** | Vertically stacked list of StepCards that appear as each step executes | The core of the screen — without real-time step rendering, the run is a black box |
| **StepCard** | Shows: tool name, status icon (spinner/green/red), latency badge (e.g. "340ms"), collapsible input args, collapsible output result | Tool name alone is not enough — users need I/O visibility to debug failures |
| **Live status bar** | Top banner: "Running step 2 of 5" with an animated progress indicator | Users must always know where in the workflow the agent is |
| **Cost ticker** | Live-updating display like "$0.0043" that increments with each step | The most-asked-about metric — must be visible at all times during execution |
| **Run timeline** | Horizontal timeline at the bottom showing each step as a colored block scaled by duration | Reveals slow steps at a glance — critical for optimization |
| **Human Approval Modal** | Full-screen modal (not a toast) with: action description, risk level badge, Approve and Reject buttons | High-stakes actions must completely interrupt the UI — a dismissible toast is not acceptable |
| **Copy run ID button** | One-click copy of the `run_id` for sharing/debugging | Support teams will ask for this — make it trivial to get |
| **Download logs button** | Exports the full run as structured JSON | Must-have for debugging after runs complete |

### Success Criteria

- Each StepCard appears on screen within **200ms** of the backend emitting the corresponding SSE event
- The cost ticker updates with every `StepCompleted` event — never shows a stale value for more than 500ms
- The Human Approval Modal renders within **1 second** of a `HumanApprovalRequired` event and is keyboard-accessible (Enter = Approve, Escape = Reject)
- After Approve is clicked: the modal closes, the paused StepCard resumes its spinner, and the next step begins within 1 second
- If the run fails mid-way: the failed StepCard turns red with an expanded error message, remaining steps show as "Cancelled" in grey
- The page is fully useful on a 1280px wide screen — no horizontal scrolling
- The run viewer works correctly for runs viewed **after** completion (replay mode from stored events in DB), not just live

### Failure States

| Scenario | What UI Must Show |
|----------|------------------|
| SSE connection drops mid-run | Yellow banner: "Connection lost — reconnecting…" with auto-retry every 3s |
| Tool call times out | StepCard shows red clock icon, "Timed out after 30s", run continues to next step if configured |
| LLM returns empty response | StepCard shows "No output from model" in orange, does not crash the viewer |
| Run rejected by human | All subsequent StepCards immediately show "Cancelled" grey state |

---

## Screen 4: Agent Marketplace

**Purpose:** A catalog of pre-built agent templates that employees can browse, preview, and fork into their own workspace. This is the network effect engine — the more agents shared, the more valuable the platform becomes.

### Must-Have Components

| Component | Description | Why It's Non-Negotiable |
|-----------|-------------|------------------------|
| **Template grid** | Card-based grid with agent name, description, tool badges, run count, and creator avatar | Cards must show enough info to decide without clicking through |
| **Search bar** | Full-text search across agent names and descriptions with instant results | Users won't scroll 50+ templates — search is the primary navigation |
| **Filter chips** | Filter by tool (e.g. "Uses Slack", "Uses GitHub"), department, or complexity | Without filters, the marketplace is unusable at scale |
| **Template detail modal** | Clicking a card shows: full description, step-by-step preview, required connectors, example output | Users must know exactly what they're getting before forking |
| **Fork button** | One click creates a copy of the template in the user's workspace, then redirects to the edit page | Zero-friction path from template to running agent |
| **"Created by" attribution** | Shows which team member created the template | Social proof drives adoption — anonymous templates get less use |
| **Required connectors checklist** | Inside the detail modal, shows which integrations are needed with green/red indicators for what's already connected | Users must know upfront if they're missing a connector before forking |

### Success Criteria

- Search returns results within **300ms** (client-side filtering on cached results is acceptable)
- Template cards load in under **1 second** on initial page load (server-rendered via RSC)
- Forking a template creates a new agent and redirects to the edit page within **2 seconds**
- Filter chips update the visible grid immediately (no page reload)
- Required connectors checklist correctly reflects the user's currently connected tools — a Slack template shows green for Slack if the user has already connected it
- Empty state (no search results) shows "No templates match — you can create one and share it" with a CTA button

---

## Screen 5: Connectors Manager

**Purpose:** The settings page where users connect their SaaS tools via OAuth, see connection status, and manage token permissions. This screen is the plumbing — it must be clear and never alarming.

### Must-Have Components

| Component | Description | Why It's Non-Negotiable |
|-----------|-------------|------------------------|
| **Connector cards grid** | One card per available integration: logo, name, connection status (Connected/Disconnected), last-used timestamp | Users must see at a glance what's connected and what isn't |
| **Connect button (OAuth flow)** | Opens OAuth popup, handles redirect, shows success/failure on return | Must be a popup, not a full redirect — preserves page context |
| **Permission scope display** | After connecting, shows exactly which scopes were granted (e.g. "Can read Slack channels, Can post messages, Cannot delete messages") | Users must see what they authorized — opaque scopes cause distrust |
| **Disconnect button** | Revokes the token and updates status immediately | Must require a 1-click confirmation: "This will break agents using Slack. Disconnect anyway?" |
| **Re-auth button** | If a token is expired, shows "Reconnect" in orange instead of "Disconnect" | Expired tokens are the #1 silent failure — must be surfaced clearly |
| **Agents using this connector** | Expandable list inside each card showing which agents depend on it | Users should know the blast radius before disconnecting |

### Success Criteria

- OAuth popup opens within **500ms** of clicking Connect
- After successful OAuth: the card updates to "Connected" with a green badge without requiring a page refresh
- Expired tokens surface as orange "Reconnect" badges — never silently fail
- The permission scope list is human-readable (not raw OAuth scope strings like `chat:write` — say "Post messages to Slack channels")
- Disconnecting a connector that has dependent agents shows the agent names in the confirmation modal

---

## Screen 6: Observability Dashboard

**Purpose:** Give team leads and admins a clear view of agent activity, cost, and health — turning the platform into a financial and operational control plane.

### Must-Have Components

| Component | Description | Why It's Non-Negotiable |
|-----------|-------------|------------------------|
| **Cost-over-time line chart** | Daily spend across all agents, filterable by agent or team, for the past 30 days | The #1 question from finance and IT: "how much is this costing?" |
| **Top agents by cost table** | Ranked list: agent name, total runs, total tokens, total cost, avg cost/run | Immediately identifies expensive or runaway agents |
| **Run success rate widget** | Percentage of runs that completed without error, last 7 days | Health indicator — if this drops below 90%, something is wrong |
| **P95 latency per agent** | Table showing the 95th percentile run duration per agent | Identifies slow agents before users complain |
| **Human approvals summary** | Count of approval requests, approval rate, average time-to-approve | Shows whether the human-in-the-loop is a bottleneck |
| **Budget alert config** | Per-team monthly budget cap with alert threshold (e.g. "Alert at 80% of $500/month") | Without budget caps, costs can spiral undetected |
| **Date range picker** | Filters all widgets by a custom date range | All metrics are useless without time context |

### Success Criteria

- All charts render server-side (RSC) with data fetched directly from Postgres aggregates — no separate analytics DB required at this scale
- The cost chart updates within **5 minutes** of a run completing (near-real-time, not real-time)
- Clicking any row in the top-agents table navigates to that agent's detail page with pre-filtered observability data
- Budget alerts send a notification (email or Slack) when the threshold is crossed — not just shown in the dashboard
- The dashboard is fully readable without any prior training — labels, units ($, ms, %), and chart titles must be self-explanatory

---

## Global UI Requirements (All Screens)

These apply to every screen in the product. A screen is not shippable if any of these are violated.

### Loading States
- Every data-fetching operation must have a skeleton loader — no blank white areas ever
- Skeleton loaders must match the approximate shape of the content they're replacing
- Streaming operations (NL builder, run viewer) must show the first chunk within 1.5s or display a "working…" indicator

### Error States
- All API errors must show a human-readable message — never a raw JSON error or stack trace
- All errors must include one of: a Retry button, a Contact Support link, or specific instructions for how to fix the issue
- Network errors must be distinguishable from application errors in the UI

### Responsiveness
- All screens must be fully functional at **1280px width minimum**
- Tables with many columns must scroll horizontally rather than truncating data
- The live run viewer must be usable on a **1080p monitor** without vertical scrolling for runs with up to 10 steps

### Accessibility
- All interactive elements must be keyboard-navigable (Tab, Enter, Escape)
- All icons used without text labels must have a `title` or `aria-label`
- Approval modals must trap keyboard focus — Tab must not escape the modal while it's open
- Color must never be the **only** signal (e.g. status must be color + icon + text, not just green/red)

### Performance
- Initial page load (Largest Contentful Paint) must be under **2.5 seconds** on a standard broadband connection
- Client-side navigation between routes must feel instant (<100ms perceived)
- SSE streams must handle **reconnection automatically** with exponential backoff up to 30s
