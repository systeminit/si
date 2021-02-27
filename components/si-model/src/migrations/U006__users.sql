CREATE TABLE users
(
    id                 bigint PRIMARY KEY,
    si_id              text UNIQUE,
    name               text                     NOT NULL,
    email              text                     NOT NULL,
    password           bytea                    NOT NULL,
    billing_account_id bigint                   NOT NULL REFERENCES billing_accounts(id),
    tenant_ids         text[]                   NOT NULL,
    obj                jsonb                    NOT NULL,
    created_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE (email, billing_account_id)
);

CREATE INDEX idx_users_tenant_ids ON "users" USING GIN ("tenant_ids");

CREATE OR REPLACE FUNCTION user_create_v1(name text,
                                          email text,
                                          pword bytea,
                                          billing_account_id text,
                                          OUT object jsonb) AS
$$
DECLARE
    id                    bigint;
    si_id                 text;
    billing_account_si_id bigint;
    tenant_ids            text[];
    created_at            timestamp with time zone;
    updated_at            timestamp with time zone;
    si_storable           jsonb;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'user:' || id INTO si_id;
    SELECT split_part(billing_account_id, ':', 2)::bigint INTO billing_account_si_id;
    SELECT ARRAY [billing_account_id, si_id] INTO tenant_ids;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;
    SELECT jsonb_build_object(
                   'typeName', 'user',
                   'tenantIds', tenant_ids,
                   'objectId', si_id,
                   'billingAccountId', billing_account_id,
                   'deleted', false,
                   'createdAt', created_at,
                   'updatedAt', updated_at
               )
    INTO si_storable;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', name,
                   'email', email,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO users
    VALUES (id,
            si_id,
            name,
            email,
            pword,
            billing_account_si_id,
            tenant_ids,
            object,
            created_at,
            updated_at);
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION user_save_v1(input_user jsonb,
                                        OUT object jsonb) AS
$$
DECLARE
    this_current_user    users%rowtype;
    this_id         bigint;
    this_tenant_ids text[];
BEGIN
    /* extract the id */
    SELECT si_id_to_primary_key_v1(input_user ->> 'id') INTO this_id;

    /* fetch the current user */
    SELECT * INTO this_current_user FROM users WHERE id = this_id;
    IF NOT FOUND THEN
        RAISE WARNING 'user id % not found', this_id;
    END IF;

    IF si_id_to_primary_key_v1(input_user -> 'siStorable' ->> 'billingAccountId') != this_current_user.billing_account_id THEN
        RAISE WARNING 'mutated billing account id; not allowed!';
    END IF;

    SELECT ARRAY(SELECT jsonb_array_elements_text(input_user -> 'siStorable' -> 'tenantIds')) INTO this_tenant_ids;

    UPDATE users
    SET name       = input_user ->> 'name',
        email      = input_user ->> 'email',
        tenant_ids = this_tenant_ids,
        obj        = input_user,
        updated_at = NOW()
    WHERE id = this_id
    RETURNING obj INTO object;
END
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION user_get_v1(si_id text, OUT object jsonb) AS
$$
DECLARE
    this_id bigint;
BEGIN
    SELECT si_id_to_primary_key_v1(si_id) INTO this_id;
    SELECT w.obj INTO object FROM users AS w WHERE id = this_id;
END
$$ LANGUAGE PLPGSQL STABLE;