-- 0001_initial_schema.sql
-- Core tables: tenants, users, agents, agent_runs, run_steps

CREATE EXTENSION IF NOT EXISTS pgcrypto;

----------------------------------------------------------------------
-- tenants
----------------------------------------------------------------------
CREATE TABLE tenants (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

----------------------------------------------------------------------
-- users
----------------------------------------------------------------------
CREATE TABLE users (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email       TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_users_email UNIQUE (email)
);

CREATE INDEX idx_users_tenant_id ON users(tenant_id);

----------------------------------------------------------------------
-- agents
----------------------------------------------------------------------
CREATE TABLE agents (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name        TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    status      TEXT NOT NULL DEFAULT 'Draft'
                CHECK (status IN ('Draft', 'Active', 'Archived')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_agents_tenant_id ON agents(tenant_id);

----------------------------------------------------------------------
-- agent_runs
----------------------------------------------------------------------
CREATE TABLE agent_runs (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id    UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    status      TEXT NOT NULL DEFAULT 'Pending'
                CHECK (status IN ('Pending', 'Running', 'WaitingForApproval', 'Succeeded', 'Failed')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_agent_runs_tenant_id ON agent_runs(tenant_id);
CREATE INDEX idx_agent_runs_agent_id  ON agent_runs(agent_id);

----------------------------------------------------------------------
-- run_steps
----------------------------------------------------------------------
CREATE TABLE run_steps (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    run_id      UUID NOT NULL REFERENCES agent_runs(id) ON DELETE CASCADE,
    label       TEXT NOT NULL,
    status      TEXT NOT NULL DEFAULT 'Pending'
                CHECK (status IN ('Pending', 'Running', 'WaitingForApproval', 'Succeeded', 'Failed')),
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_run_steps_tenant_id ON run_steps(tenant_id);
CREATE INDEX idx_run_steps_run_id    ON run_steps(run_id);
