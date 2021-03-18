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
    reverted           bool                     NOT NULL DEFAULT false,
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
                               tenant_ids, obj, reverted, created_at, updated_at)
    VALUES (this_id, si_id, this_name, this_billing_account_id, this_organization_id, this_workspace_id,
            this_change_set_id, tenant_ids, object, false, created_at, updated_at);

END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION edit_session_save_session_v1(this_si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id  bigint;
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
    ON CONFLICT(id, change_set_id) DO UPDATE
        SET obj        = excluded.obj,
            updated_at = NOW();
END
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION edit_session_save_v1(input_edit_session jsonb,
                                                OUT object jsonb) AS
$$
DECLARE
    this_current edit_sessions%rowtype;
    this_id      bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_edit_session ->> 'id') INTO this_id;

    SELECT * INTO this_current FROM edit_sessions WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'edit_session id % not found', this_id;
    END IF;

    /* bail if it is a tenancy violation */
    IF si_id_to_primary_key_v1(input_edit_session -> 'siStorable' ->> 'billingAccountId') !=
       this_current.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    UPDATE edit_sessions
    SET name       = input_edit_session ->> 'name',
        reverted   = (input_edit_session ->> 'reverted')::bool,
        obj        = input_edit_session,
        updated_at = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION edit_session_revert_v1(this_edit_session_si_id text, OUT object jsonb) AS
$$
DECLARE
    current_obj           jsonb;
    this_edit_session_id  bigint;
    this_edit_session_obj jsonb;
BEGIN
    SELECT si_id_to_primary_key_v1(this_edit_session_si_id) INTO this_edit_session_id;
    SELECT obj INTO current_obj FROM edit_sessions WHERE id = this_edit_session_id;
    SELECT jsonb_set(current_obj, '{reverted}', 'true'::jsonb) INTO this_edit_session_obj;

    UPDATE edit_sessions
    SET reverted   = true,
        obj        = this_edit_session_obj,
        updated_at = now()
    WHERE id = this_edit_session_id;

    UPDATE ops
    SET obj        = jsonb_set(obj, '{siOp, skip}', 'true'::jsonb),
        updated_at = now()
    WHERE edit_session_id = this_edit_session_id
    RETURNING ops.obj INTO object;
END;
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
