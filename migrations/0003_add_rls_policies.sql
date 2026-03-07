-- 0003_add_rls_policies.sql
-- Tenant isolation via row-level security.
--
-- The application layer must call:
--   SET LOCAL app.tenant_id = '<uuid>';
-- on every connection/transaction before executing queries.

----------------------------------------------------------------------
-- Enable RLS on all tables
----------------------------------------------------------------------
ALTER TABLE tenants        ENABLE ROW LEVEL SECURITY;
ALTER TABLE users          ENABLE ROW LEVEL SECURITY;
ALTER TABLE agents         ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent_runs     ENABLE ROW LEVEL SECURITY;
ALTER TABLE run_steps      ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent_versions ENABLE ROW LEVEL SECURITY;

----------------------------------------------------------------------
-- Force RLS for the table owner role too (without this, the superuser
-- or table owner silently bypasses all policies).
----------------------------------------------------------------------
ALTER TABLE tenants        FORCE ROW LEVEL SECURITY;
ALTER TABLE users          FORCE ROW LEVEL SECURITY;
ALTER TABLE agents         FORCE ROW LEVEL SECURITY;
ALTER TABLE agent_runs     FORCE ROW LEVEL SECURITY;
ALTER TABLE run_steps      FORCE ROW LEVEL SECURITY;
ALTER TABLE agent_versions FORCE ROW LEVEL SECURITY;

----------------------------------------------------------------------
-- tenants — uses "id" instead of tenant_id
----------------------------------------------------------------------
CREATE POLICY tenant_isolation ON tenants
    USING (id = current_setting('app.tenant_id')::uuid);

----------------------------------------------------------------------
-- users
----------------------------------------------------------------------
CREATE POLICY tenant_isolation ON users
    USING (tenant_id = current_setting('app.tenant_id')::uuid);

----------------------------------------------------------------------
-- agents
----------------------------------------------------------------------
CREATE POLICY tenant_isolation ON agents
    USING (tenant_id = current_setting('app.tenant_id')::uuid);

----------------------------------------------------------------------
-- agent_runs
----------------------------------------------------------------------
CREATE POLICY tenant_isolation ON agent_runs
    USING (tenant_id = current_setting('app.tenant_id')::uuid);

----------------------------------------------------------------------
-- run_steps
----------------------------------------------------------------------
CREATE POLICY tenant_isolation ON run_steps
    USING (tenant_id = current_setting('app.tenant_id')::uuid);

----------------------------------------------------------------------
-- agent_versions
----------------------------------------------------------------------
CREATE POLICY tenant_isolation ON agent_versions
    USING (tenant_id = current_setting('app.tenant_id')::uuid);
