CREATE TABLE edit_sessions
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    name               text                     NOT NULL,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    change_set_id      bigint                   NOT NULL REFERENCES change_sets (id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    canceled           bool                     NOT NULL DEFAULT false,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_edit_sessions_tenant_ids ON "edit_sessions" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION edit_session_create_v1(this_name text,
                                                  this_note text,
                                                  this_change_set_si_id text,
                                                  si_workspace_id text,
                                                  OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    this_change_set_id      bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'editSession:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at);

    SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', this_name,
                   'note', this_note,
                   'canceled', false,
                   'saved', false,
                   'changeSetId', this_change_set_si_id,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO edit_sessions (id, si_id, name, billing_account_id, organization_id, workspace_id, change_set_id,
                               tenant_ids, obj, canceled, created_at, updated_at)
    VALUES (this_id, si_id, this_name, this_billing_account_id, this_organization_id, this_workspace_id,
            this_change_set_id, tenant_ids, object, false, created_at, updated_at);

END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION edit_session_cancel_v1(this_si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(this_si_id) INTO this_id;

    UPDATE edit_sessions SET obj = jsonb_set(obj, '{saved}', 'false'::jsonb) WHERE id = this_id;
    UPDATE edit_sessions SET obj = jsonb_set(obj, '{canceled}', 'true'::jsonb) WHERE id = this_id;
    UPDATE edit_sessions SET canceled = true WHERE id = this_id;
    SELECT obj INTO object FROM edit_sessions WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION edit_session_save_session_v1(this_si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(this_si_id) INTO this_id;

    UPDATE edit_sessions SET obj = jsonb_set(obj, '{saved}', 'true'::jsonb) WHERE id = this_id;
    SELECT obj INTO object FROM edit_sessions WHERE id = this_id;

    INSERT INTO entities_change_set_projection (id, obj, change_set_id, tenant_ids, created_at)
    SELECT entities_edit_session_projection.id,
           entities_edit_session_projection.obj,
           entities_edit_session_projection.change_set_id,
           entities_edit_session_projection.tenant_ids,
           entities_edit_session_projection.created_at
    FROM entities_edit_session_projection
    WHERE entities_edit_session_projection.edit_session_id = this_id
      AND NOT (
            entities_edit_session_projection.obj -> 'siStorable' -> 'deleted' = 'true' AND
            entities_edit_session_projection.id NOT IN (SELECT entities_change_set_projection.id
                                                        FROM entities_change_set_projection
                                                        WHERE entities_change_set_projection.change_set_id =
                                                              entities_edit_session_projection.change_set_id) AND
            entities_edit_session_projection.id NOT IN (SELECT entities_head.id
                                                        FROM entities_head
                                                        WHERE entities_head.id =
                                                              entities_edit_session_projection.id)
        )
    ON CONFLICT(id, change_set_id) DO UPDATE
        SET obj        = excluded.obj,
            updated_at = NOW();

    INSERT INTO qualifications_change_set_projection (id, obj, qualified, change_set_id, tenant_ids, created_at)
    SELECT qualifications_edit_session_projection.id,
           qualifications_edit_session_projection.obj,
           qualifications_edit_session_projection.qualified,
           qualifications_edit_session_projection.change_set_id,
           qualifications_edit_session_projection.tenant_ids,
           qualifications_edit_session_projection.created_at
    FROM qualifications_edit_session_projection
    WHERE qualifications_edit_session_projection.edit_session_id = this_id
    ON CONFLICT(id, change_set_id) DO UPDATE
        SET obj        = excluded.obj,
            updated_at = NOW();
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION edit_session_get_v1(si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    SELECT w.obj INTO object FROM edit_sessions AS w WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL STABLE;
