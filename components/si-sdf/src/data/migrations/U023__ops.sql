CREATE TABLE ops
(
    id                      bigint PRIMARY KEY,
    si_id                   text UNIQUE,
    to_si_id                text,
    op_type                 text,
    obj jsonb NOT NULL,
    billing_account_id      bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id         bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id            bigint                   NOT NULL REFERENCES workspaces (id),
    tenant_ids              text[]                   NOT NULL,
    epoch                   bigint                   NOT NULL,
    update_count            bigint                   NOT NULL,
    change_set_id           bigint                   NOT NULL REFERENCES change_sets (id),
    change_set_epoch        bigint                   NOT NULL,
    change_set_update_count bigint                   NOT NULL,
    edit_session_id         bigint                   NOT NULL REFERENCES edit_sessions (id),
    created_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ops_tenant_ids ON "ops" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION op_create_v1(this_op_type text,
                                        this_to_si_id text,
                                        object_base jsonb,
                                        this_override_system text,
                                        this_change_set_si_id text,
                                        this_edit_session_si_id text,
                                        this_si_change_set_event text,
                                        si_workspace_id text,
                                        this_workspace_epoch bigint,
                                        this_workspace_update_count bigint,
                                        this_change_set_epoch bigint,
                                        this_change_set_update_count bigint,
                                        OUT object jsonb) AS
$$
DECLARE
    this_id                    bigint;
    si_id                      text;
    this_workspace_id          bigint;
    this_organization_id       bigint;
    this_billing_account_id    bigint;
    this_change_set_id         bigint;
    this_edit_session_id       bigint;
    tenant_ids                 text[];
    created_at                 timestamp with time zone;
    updated_at                 timestamp with time zone;
    si_storable                jsonb;
    si_change_set              jsonb;
    si_change_set_update_clock jsonb;
    si_op                      jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT this_op_type || ':' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at, this_workspace_epoch,
                               this_workspace_update_count);

    SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;
    SELECT si_id_to_primary_key_v1(this_edit_session_si_id) INTO this_edit_session_id;

    SELECT jsonb_build_object(
                   'epoch', this_change_set_epoch,
                   'updateCount', this_change_set_update_count
               )
    INTO si_change_set_update_clock;

    SELECT jsonb_build_object('changeSetId', this_change_set_si_id,
                              'editSessionId', this_edit_session_si_id,
                              'event', this_si_change_set_event,
                              'orderClock', si_change_set_update_clock
               )
    INTO si_change_set;

    SELECT jsonb_build_object(
        'skip', false,
        'overrideSystem', this_override_system
    ) INTO si_op;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'toId', this_to_si_id,
                   'siOp', si_op,
                   'siChangeSet', si_change_set,
                   'siStorable', si_storable
               ) || object_base
    INTO object;

    INSERT INTO ops (id, si_id, to_si_id, op_type, obj, billing_account_id, organization_id, workspace_id, tenant_ids, epoch,
                     update_count, change_set_id, change_set_epoch, change_set_update_count, edit_session_id, created_at, updated_at)
    VALUES (this_id, si_id, this_to_si_id, this_op_type, object, this_billing_account_id, this_organization_id,
            this_workspace_id, tenant_ids, this_workspace_epoch, this_workspace_update_count, this_change_set_id,
            this_change_set_epoch, this_change_set_update_count, this_edit_session_id, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
