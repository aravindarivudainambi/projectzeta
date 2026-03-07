-- 0004_add_approval_requests.sql
-- Human-in-the-loop approval checkpoints for agent runs.

CREATE TABLE approval_requests (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    run_id      UUID NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
    step_id     UUID NOT NULL,
    action      TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'Pending'
                CHECK (status IN ('Pending', 'Approved', 'Rejected')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    decided_at  TIMESTAMPTZ
);

CREATE INDEX idx_approval_requests_tenant_id ON approval_requests(tenant_id);
CREATE INDEX idx_approval_requests_run_id    ON approval_requests(run_id);

-- RLS for tenant isolation (matching 0003 pattern)
ALTER TABLE approval_requests ENABLE ROW LEVEL SECURITY;
ALTER TABLE approval_requests FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON approval_requests
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
