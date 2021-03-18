CREATE TABLE node_positions
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    node_id            bigint                   NOT NULL REFERENCES nodes (id),
    context_id         text                     NOT NULL,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (node_id, context_id)
);

CREATE OR REPLACE FUNCTION node_position_create_v1(si_node_id text,
                                                   context_id text,
                                                   x text,
                                                   y text,
                                                   si_workspace_id text,
                                                   OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    this_node_id            bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'nodePosition:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at);

    SELECT si_id_to_primary_key_v1(si_node_id) INTO this_node_id;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'nodeId', si_node_id,
                   'contextId', context_id,
                   'x', x,
                   'y', y,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO node_positions(id, si_id, node_id, context_id, billing_account_id, organization_id, workspace_id,
                               tenant_ids, obj, created_at, updated_at)
    VALUES (this_id, si_id, this_node_id, context_id, this_billing_account_id, this_organization_id, this_workspace_id,
            tenant_ids, object, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION node_position_update_v1(si_node_id text,
                                                   this_context_id text,
                                                   x text,
                                                   y text,
                                                   OUT object jsonb) AS
$$
DECLARE
    this_node_id bigint;
    this_current jsonb;
BEGIN
    SELECT si_id_to_primary_key_v1(si_node_id) INTO this_node_id;

    SELECT obj INTO this_current FROM node_positions WHERE node_id = this_node_id AND context_id = this_context_id;
    SELECT jsonb_set(this_current, '{x}', to_jsonb(x)) INTO this_current;
    SELECT jsonb_set(this_current, '{y}', to_jsonb(y)) INTO this_current;

    UPDATE node_positions
    SET obj        = this_current,
        updated_at = NOW()
    WHERE node_id = this_node_id
      AND context_id = this_context_id
    RETURNING obj INTO object;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION node_position_create_or_update_v1(si_node_id text,
                                                             this_context_id text,
                                                             x text,
                                                             y text,
                                                             si_workspace_id text,
                                                             OUT object jsonb) AS
$$
DECLARE
    this_node_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_node_id) INTO this_node_id;

    PERFORM (id) FROM node_positions WHERE node_id = this_node_id AND context_id = this_context_id;

    IF FOUND THEN
        SELECT node_position_update_v1(
                       si_node_id,
                       this_context_id,
                       x,
                       y
                   )
        INTO object;
    ELSE
        SELECT node_position_create_v1(si_node_id,
                                       this_context_id,
                                       x,
                                       y,
                                       si_workspace_id
                   )
        INTO object;
    END IF;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION node_position_save_v1(input_node_position jsonb,
                                                 OUT object jsonb) AS
$$
DECLARE
    this_current node_positions%rowtype;
    this_id      bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_node_position ->> 'id') INTO this_id;

    SELECT * INTO this_current FROM node_positions WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'node position id % not found', this_id;
    END IF;

    /* bail if it is a tenancy violation */
    IF si_id_to_primary_key_v1(input_node_position -> 'siStorable' ->> 'billingAccountId') !=
       this_current.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    UPDATE node_positions
    SET obj          = input_node_position,
        updated_at   = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;
