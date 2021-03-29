CREATE TABLE qualifications
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    entity_id          bigint                   NOT NULL REFERENCES entities (id),
    name               text                     NOT NULL,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    tenant_ids         text[]                   NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (entity_id, name)
);

CREATE TABLE qualifications_head
(
    id         bigint PRIMARY KEY REFERENCES qualifications (id),
    obj        jsonb                    NOT NULL,
    qualified  bool                     NOT NULL,
    tenant_ids text[]                   NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE TABLE qualifications_change_set_projection
(
    id            bigint REFERENCES qualifications (id),
    obj           jsonb                    NOT NULL,
    qualified     bool                     NOT NULL,
    change_set_id bigint                   NOT NULL REFERENCES change_sets (id),
    tenant_ids    text[]                   NOT NULL,
    created_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id)
);

CREATE TABLE qualifications_edit_session_projection
(
    id              bigint REFERENCES qualifications (id),
    obj             jsonb                    NOT NULL,
    qualified       bool                     NOT NULL,
    change_set_id   bigint                   NOT NULL REFERENCES change_sets (id),
    edit_session_id bigint                   NOT NULL REFERENCES edit_sessions (id),
    tenant_ids      text[]                   NOT NULL,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (id, change_set_id, edit_session_id)
);

CREATE OR REPLACE FUNCTION qualification_create_or_update_v1(
    this_entity_si_id text,
    this_name text,
    this_qualified bool,
    this_output text,
    this_error text,
    this_change_set_si_id text,
    this_edit_session_si_id text,
    si_workspace_id text,
    OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_entity_id          bigint;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    this_change_set_id      bigint;
    this_edit_session_id    bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
    si_change_set           jsonb;
BEGIN
    SELECT si_id_to_primary_key_v1(this_entity_si_id) INTO this_entity_id;
    SELECT id INTO this_id FROM qualifications WHERE entity_id = this_entity_id AND name = this_name;
    IF NOT FOUND THEN
        RAISE NOTICE 'we had to generate our own id';
        SELECT next_si_id_v1() INTO this_id;
    END IF;

    SELECT 'qualification:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at);

    SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;
    SELECT si_id_to_primary_key_v1(this_edit_session_si_id) INTO this_edit_session_id;

    SELECT jsonb_build_object('changeSetId', this_change_set_si_id,
                              'editSessionId', this_edit_session_si_id)
    INTO si_change_set;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'entityId', this_entity_si_id,
                   'name', this_name,
                   'qualified', this_qualified,
                   'output', this_output,
                   'error', this_error,
                   'siStorable', si_storable,
                   'siChangeSet', si_change_set
               )
    INTO object;

    -- We don't care if a qualification already exists! --
    INSERT INTO qualifications (id, si_id, entity_id, name, billing_account_id, organization_id, workspace_id,
                                tenant_ids)
    VALUES (this_id, si_id, this_entity_id, this_name, this_billing_account_id, this_organization_id, this_workspace_id,
            tenant_ids)
    ON CONFLICT DO NOTHING;

    INSERT INTO qualifications_edit_session_projection (id, obj, qualified, change_set_id, edit_session_id, tenant_ids)
    VALUES (this_id, object, this_qualified, this_change_set_id, this_edit_session_id, tenant_ids)
    ON CONFLICT (id, change_set_id, edit_session_id) DO UPDATE SET obj = object, qualified = this_qualified, updated_at = NOW();
END;
$$ LANGUAGE PLPGSQL VOLATILE;
