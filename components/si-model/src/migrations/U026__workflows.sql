CREATE TABLE workflows
(
    id         bigint PRIMARY KEY,
    si_id      text UNIQUE,
    name       text UNIQUE              NOT NULL,
    obj        jsonb                    NOT NULL,
    tenant_ids text[]                   NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (name)
);

CREATE OR REPLACE FUNCTION workflow_create_or_update_v1(data jsonb,
                                                        OUT object jsonb) AS
$$
DECLARE
    this_name      text;
    this_id        bigint;
    si_id          text;
    our_tenant_ids text[];
    created_at     timestamp with time zone;
    updated_at     timestamp with time zone;
    si_storable    jsonb;
BEGIN
    SELECT data ->> 'name' INTO this_name;
    SELECT id INTO this_id FROM workflows WHERE name = this_name;
    IF NOT FOUND THEN
        SELECT next_si_id_v1() INTO this_id;
    END IF;

    SELECT 'workflow:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT jsonb_build_object(
                   'typeName', 'workflow',
                   'objectId', si_id,
                   'deleted', false,
                   'createdAt', created_at,
                   'updatedAt', updated_at
               )
    INTO si_storable;


    SELECT ARRAY [si_id]
    INTO our_tenant_ids;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', this_name,
                   'data', data,
                   'siStorable', si_storable
               )
    INTO object;

    -- We don't care if a workflow already exists! --
    INSERT INTO workflows (id, si_id, name, obj, tenant_ids, created_at, updated_at)
    VALUES (this_id, si_id, this_name, object, our_tenant_ids, created_at, updated_at)
    ON CONFLICT (id) DO UPDATE SET obj = object, updated_at = NOW();
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE TABLE workflow_runs
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    workflow_id        bigint                   NOT NULL REFERENCES workflows (id),
    dry_run            bool                     NOT NULL DEFAULT false,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION workflow_run_create_v1(this_workflow_si_id text,
                                                  workflow_name text,
                                                  data jsonb,
                                                  ctx jsonb,
                                                  state text,
                                                  start_timestamp text,
                                                  start_unix_timestamp bigint,
                                                  OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workflow_id        bigint;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
    si_workspace_id         text;
    this_dry_run            bool;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'workflowRun:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT si_id_to_primary_key_v1(this_workflow_si_id) INTO this_workflow_id;

    SELECT ctx -> 'workspace' ->> 'id' INTO si_workspace_id;
    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at);

    SELECT (ctx ->> 'dryRun')::bool INTO this_dry_run;
    SELECT jsonb_build_object(
                   'id', si_id,
                   'startTimestamp', start_timestamp,
                   'startUnixTimestamp', start_unix_timestamp,
                   'state', state,
                   'workflowId', this_workflow_si_id,
                   'workflowName', workflow_name,
                   'data', data,
                   'ctx', ctx,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO workflow_runs (id, si_id, workflow_id, dry_run, billing_account_id, organization_id, workspace_id,
                               tenant_ids, obj,
                               created_at, updated_at)
    VALUES (this_id, si_id, this_workflow_id, this_dry_run, this_billing_account_id, this_organization_id,
            this_workspace_id, tenant_ids,
            object, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION workflow_run_save_v1(input jsonb,
                                                     OUT object jsonb) AS
$$
DECLARE
    this_current workflow_runs%rowtype;
    this_id      bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input ->> 'id') INTO this_id;

    SELECT * INTO this_current FROM workflow_runs WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'workflow run id % not found', this_id;
    END IF;

    /* bail if it is a tenancy violation */
    IF si_id_to_primary_key_v1(input -> 'siStorable' ->> 'billingAccountId') !=
       this_current.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    UPDATE workflow_runs
    SET obj        = input,
        updated_at = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;


CREATE TABLE workflow_run_steps
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    workflow_run_id    bigint                   NOT NULL REFERENCES workflow_runs (id),
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION workflow_run_step_create_v1(this_workflow_run_si_id text,
                                                       this_step jsonb,
                                                       this_state text,
                                                       start_timestamp text,
                                                       start_unix_timestamp bigint,
                                                       this_workspace_si_id text,
                                                       OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workflow_run_id    bigint;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'workflowRunStep:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, this_workspace_si_id, created_at, updated_at);

    SELECT si_id_to_primary_key_v1(this_workflow_run_si_id) INTO this_workflow_run_id;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'workflowRunId', this_workflow_run_si_id,
                   'startTimestamp', start_timestamp,
                   'startUnixTimestamp', start_unix_timestamp,
                   'state', this_state,
                   'step', this_step,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO workflow_run_steps (id, si_id, workflow_run_id, billing_account_id, organization_id, workspace_id,
                                    tenant_ids, obj, created_at, updated_at)
    VALUES (this_id, si_id, this_workflow_run_id, this_billing_account_id, this_organization_id, this_workspace_id,
            tenant_ids, object, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION workflow_run_step_save_v1(input jsonb,
                                                     OUT object jsonb) AS
$$
DECLARE
    this_current workflow_run_steps%rowtype;
    this_id      bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input ->> 'id') INTO this_id;

    SELECT * INTO this_current FROM workflow_run_steps WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'workflow run step id % not found', this_id;
    END IF;

    /* bail if it is a tenancy violation */
    IF si_id_to_primary_key_v1(input -> 'siStorable' ->> 'billingAccountId') !=
       this_current.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    UPDATE workflow_run_steps
    SET obj        = input,
        updated_at = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;

CREATE TABLE workflow_run_step_entities
(
    id                   bigint PRIMARY KEY,
    si_id                text UNIQUE,
    workflow_run_step_id bigint                   NOT NULL REFERENCES workflow_run_steps (id),
    entity_id            bigint                   NOT NULL REFERENCES entities (id),
    billing_account_id   bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id      bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id         bigint                   NOT NULL REFERENCES workspaces (id),
    tenant_ids           text[]                   NOT NULL,
    obj                  jsonb                    NOT NULL,
    created_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at           TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION workflow_run_step_entity_create_v1(this_workflow_run_si_id text,
                                                              this_workflow_run_step_si_id text,
                                                              this_entity_si_id text,
                                                              this_state text,
                                                              start_timestamp text,
                                                              start_unix_timestamp bigint,
                                                              this_workspace_si_id text,
                                                              OUT object jsonb) AS
$$
DECLARE
    this_id                   bigint;
    si_id                     text;
    this_entity_id            bigint;
    this_workflow_run_step_id bigint;
    this_workspace_id         bigint;
    this_organization_id      bigint;
    this_billing_account_id   bigint;
    tenant_ids                text[];
    created_at                timestamp with time zone;
    updated_at                timestamp with time zone;
    si_storable               jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'workflowRunStepEntity:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, this_workspace_si_id, created_at, updated_at);

    SELECT si_id_to_primary_key_v1(this_workflow_run_step_si_id) INTO this_workflow_run_step_id;
    SELECT si_id_to_primary_key_v1(this_entity_si_id) INTO this_entity_id;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'entityId', this_entity_si_id,
                   'workflowRunId', this_workflow_run_si_id,
                   'workflowRunStepId', this_workflow_run_step_si_id,
                   'startTimestamp', start_timestamp,
                   'startUnixTimestamp', start_unix_timestamp,
                   'state', this_state,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO workflow_run_step_entities (id, si_id, workflow_run_step_id, entity_id, billing_account_id,
                                            organization_id, workspace_id, tenant_ids, obj, created_at, updated_at)
    VALUES (this_id, si_id, this_workflow_run_step_id, this_entity_id, this_billing_account_id, this_organization_id,
            this_workspace_id, tenant_ids, object, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION workflow_run_step_entity_save_v1(input jsonb,
                                                     OUT object jsonb) AS
$$
DECLARE
    this_current workflow_run_step_entities%rowtype;
    this_id      bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input ->> 'id') INTO this_id;

    SELECT * INTO this_current FROM workflow_run_step_entities WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'workflow run step id % not found', this_id;
    END IF;

    /* bail if it is a tenancy violation */
    IF si_id_to_primary_key_v1(input -> 'siStorable' ->> 'billingAccountId') !=
       this_current.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    UPDATE workflow_run_step_entities
    SET obj        = input,
        updated_at = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;
