-- 0002_add_agent_versions.sql
-- Behavioral versioning for agents

CREATE TABLE agent_versions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id   UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id    UUID NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    version     INTEGER NOT NULL,
    snapshot    JSONB NOT NULL,
    is_active   BOOLEAN NOT NULL DEFAULT false,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_agent_versions_agent_version UNIQUE (agent_id, version)
);

CREATE INDEX idx_agent_versions_tenant_id ON agent_versions(tenant_id);
CREATE INDEX idx_agent_versions_agent_id  ON agent_versions(agent_id);

-- Only one active version per agent at a time.
CREATE UNIQUE INDEX idx_agent_versions_active
    ON agent_versions(agent_id)
    WHERE is_active = true;
