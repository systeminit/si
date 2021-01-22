CREATE TABLE change_sets
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    name               text                     NOT NULL,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    epoch              bigint                   NOT NULL,
    update_count       bigint                   NOT NULL,
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (name, workspace_id)
);

CREATE INDEX idx_change_sets_tenant_ids ON "change_sets" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION change_set_create_v1(this_name text,
                                                this_note text,
                                                this_status text,
                                                si_workspace_id text,
                                                this_epoch bigint,
                                                this_update_count bigint,
                                                OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'changeSet:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at, this_epoch, this_update_count);

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', this_name,
                   'note', this_note,
                   'status', this_status,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO change_sets (id, si_id, name, billing_account_id, organization_id, workspace_id, epoch,
                             update_count, tenant_ids, obj, created_at, updated_at)
    VALUES (this_id, si_id, this_name, this_billing_account_id, this_organization_id,
            this_workspace_id, this_epoch, this_update_count, tenant_ids, object, created_at, updated_at);

    INSERT INTO update_clocks VALUES (DEFAULT, si_id, DEFAULT, DEFAULT);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION change_set_save_v1(input_change_set jsonb,
                                              OUT object jsonb) AS
$$
DECLARE
    this_current change_sets%rowtype;
    this_id      bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_change_set ->> 'id') INTO this_id;

    SELECT * INTO this_current FROM change_sets WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'change_set id % not found', this_id;
    END IF;

    /* bail if it is a tenancy violation */
    IF si_id_to_primary_key_v1(input_change_set -> 'siStorable' ->> 'billingAccountId') !=
       this_current.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    UPDATE change_sets
    SET name         = input_change_set ->> 'name',
        epoch        = (input_change_set -> 'siStorable' -> 'updateClock' ->> 'epoch')::bigint,
        update_count = (input_change_set -> 'siStorable' -> 'updateClock' ->> 'updateCount')::bigint,
        obj          = input_change_set,
        updated_at   = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION change_set_get_v1(si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    SELECT w.obj INTO object FROM change_sets AS w WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL STABLE;

CREATE TABLE change_set_participants
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    change_set_id      bigint                   NOT NULL REFERENCES change_sets (id),
    object_si_id       text                     NOT NULL,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    epoch              bigint                   NOT NULL,
    update_count       bigint                   NOT NULL,
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (change_set_id, object_si_id)
);

CREATE OR REPLACE FUNCTION change_set_participant_create_v1(this_change_set_si_id text,
                                                            this_object_si_id text,
                                                            si_workspace_id text,
                                                            this_epoch bigint,
                                                            this_update_count bigint,
                                                            OUT object jsonb) AS
$$
DECLARE
    this_id                 bigint;
    si_id                   text;
    this_change_set_id      bigint;
    this_workspace_id       bigint;
    this_organization_id    bigint;
    this_billing_account_id bigint;
    tenant_ids              text[];
    created_at              timestamp with time zone;
    updated_at              timestamp with time zone;
    si_storable             jsonb;
BEGIN
    SELECT next_si_id_v1() INTO this_id;
    SELECT 'changeSetParticipant:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at, this_epoch, this_update_count);

    SELECT jsonb_build_object(
                   'id', si_id,
                   'changeSetId', this_change_set_si_id,
                   'objectId', this_object_si_id,
                   'siStorable', si_storable
               )
    INTO object;

    --- Should we be validating the object ID? I bet we should.
    INSERT INTO change_set_participants (id, si_id, change_set_id, object_si_id, billing_account_id, organization_id,
                                         workspace_id, epoch, update_count, tenant_ids, obj, created_at, updated_at)
    VALUES (this_id, si_id, this_change_set_id, this_object_si_id, this_billing_account_id, this_organization_id,
            this_workspace_id, this_epoch, this_update_count, tenant_ids, object, created_at, updated_at);

END;
$$ LANGUAGE PLPGSQL VOLATILE;