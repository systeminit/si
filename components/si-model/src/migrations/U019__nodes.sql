CREATE TABLE nodes
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    object_type        text                     NOT NULL,
    object_si_id       text                     NOT NULL,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION node_create_v1(this_object_type text,
                                          si_workspace_id text,
                                          OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    this_positions          jsonb;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'node:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at);

    SELECT '{}'::jsonb INTO this_positions;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'objectId', 'sovietRussiaPoopCanoeOwnsYou',
                   'objectType', this_object_type,
                   'positions', this_positions,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO nodes (id, si_id, object_type, object_si_id, billing_account_id, organization_id, workspace_id,
                       tenant_ids, obj, created_at, updated_at)
    VALUES (this_id, si_id, this_object_type, 'sovietRussiaPoopCanoeOwnsYou', this_billing_account_id,
            this_organization_id, this_workspace_id, tenant_ids, object, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION node_update_object_id_v1(this_si_id text, this_object_si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id          bigint;
    current_obj      jsonb;
    this_updated_obj jsonb;
BEGIN
    SELECT si_id_to_primary_key_v1(this_si_id) INTO this_id;
    SELECT obj FROM nodes WHERE id = this_id INTO current_obj;
    SELECT jsonb_set(current_obj, '{objectId}', to_jsonb(this_object_si_id)) INTO this_updated_obj;

    UPDATE nodes
    SET object_si_id = this_object_si_id,
        obj          = this_updated_obj
    WHERE id = this_id
    RETURNING obj INTO object;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION node_save_v1(input_node jsonb,
                                        OUT object jsonb) AS
$$
DECLARE
    this_current nodes%rowtype;
    this_id      bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_node ->> 'id') INTO this_id;

    SELECT * INTO this_current FROM nodes WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'node id % not found', this_id;
    END IF;

    /* bail if it is a tenancy violation */
    IF si_id_to_primary_key_v1(input_node -> 'siStorable' ->> 'billingAccountId') !=
       this_current.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    UPDATE nodes
    SET obj        = input_node,
        updated_at = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION node_get_v1(si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    SELECT w.obj INTO object FROM nodes AS w WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL STABLE;
