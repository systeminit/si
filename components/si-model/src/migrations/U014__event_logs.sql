CREATE TABLE event_logs
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts (id),
    organization_id    bigint                   NOT NULL REFERENCES organizations (id),
    workspace_id       bigint                   NOT NULL REFERENCES workspaces (id),
    event_id           bigint                   NOT NULL REFERENCES events (id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_event_logs_tenant_ids ON "event_logs" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION event_log_create_v1(this_message text,
                                               this_payload jsonb,
                                               this_event_si_id text,
                                               this_level text,
                                               this_timestamp text,
                                               this_unix_timestamp bigint,
                                               si_workspace_id text,
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
    SELECT 'eventLog:' || this_id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT our_si_storable, our_organization_id, our_billing_account_id, our_workspace_id, our_tenant_ids
    INTO si_storable, this_organization_id, this_billing_account_id, this_workspace_id, tenant_ids
    FROM si_storable_create_v1(si_id, si_workspace_id, created_at, updated_at);

    SELECT jsonb_build_object(
                   'id', si_id,
                   'message', this_message,
                   'payload', this_payload,
                   'level', this_level,
                   'eventId', this_event_si_id,
                   'hasOutputLine', false,
                   'unixTimestamp', this_unix_timestamp,
                   'timestamp', this_timestamp,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO event_logs (id, si_id, billing_account_id, organization_id, workspace_id, event_id,
                            tenant_ids, obj, created_at, updated_at)
    VALUES (this_id, si_id, this_billing_account_id, this_organization_id, this_workspace_id,
            si_id_to_primary_key_v1(this_event_si_id), tenant_ids, object, created_at,
            updated_at);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION event_log_save_v1(input_event_log jsonb,
                                         OUT object jsonb) AS
$$
DECLARE
    this_current event_logs%rowtype;
    this_id            bigint;
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_event_log ->> 'id') INTO this_id;

    SELECT * INTO this_current FROM event_logs WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'event_log id % not found', this_id;
    END IF;

    /* bail if it is a tenancy violation */
    IF si_id_to_primary_key_v1(input_event_log -> 'siStorable' ->> 'billingAccountId') !=
       this_current.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    UPDATE event_logs
    SET obj        = input_event_log,
        updated_at = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION event_log_get_v1(si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    SELECT w.obj INTO object FROM event_logs AS w WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL STABLE;
