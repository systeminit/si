CREATE TABLE encrypted_secrets
(
    pk                       ident primary key                 default ident_create_v1(),
    id                       ident                    not null default ident_create_v1(),
    tenancy_workspace_pk     ident,
    visibility_change_set_pk ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at    timestamp with time zone,
    created_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    created_by               ident,
    updated_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_by               ident,
    name                     text                     NOT NULL,
    definition               text                     NOT NULL,
    description              text,
    key_pair_pk              ident                    NOT NULL,
    nonce                    bytea                    NOT NULL,
    donkey_hash              bytea                    NOT NULL,
    crypted                  text                     NOT NULL,
    version                  text                     NOT NULL,
    algorithm                text                     NOT NULL
);
SELECT standard_model_table_constraints_v1('encrypted_secrets');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('encrypted_secrets', 'model', 'encrypted_secret', 'Encrypted Secret');

-- The Rust type `Secret` will use this view as its source-of-truth "table" as
-- it is a read-only subset of encrypted_secrets data
CREATE VIEW secrets AS
SELECT pk,
       id,
       tenancy_workspace_pk,
       visibility_change_set_pk,
       visibility_deleted_at,
       key_pair_pk,
       created_at,
       created_by,
       updated_at,
       updated_by,
       name,
       definition,
       description
FROM encrypted_secrets;

-- We need to create the following tenancy and visibility related functions by hand
-- because we're trying to pretend that the secrets view is a "normal" standard model
-- table.
CREATE OR REPLACE FUNCTION in_tenancy_v1(
    this_tenancy jsonb,
    record_to_check secrets
)
    RETURNS bool
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
AS
$$
SELECT in_tenancy_v1(
               this_tenancy,
               record_to_check.tenancy_workspace_pk
           )
$$;

CREATE OR REPLACE FUNCTION is_visible_v1(
    this_visibility jsonb,
    record_to_check secrets
)
    RETURNS bool
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
AS
$$
SELECT is_visible_v1(
               this_visibility,
               record_to_check.visibility_change_set_pk,
               record_to_check.visibility_deleted_at
           )
$$;

CREATE OR REPLACE FUNCTION in_tenancy_and_visible_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    record_to_check secrets
)
    RETURNS bool
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE CALLED ON NULL INPUT
AS
$$
SELECT in_tenancy_v1(
               this_tenancy,
               record_to_check.tenancy_workspace_pk
           )
           AND is_visible_v1(
               this_visibility,
               record_to_check.visibility_change_set_pk,
               record_to_check.visibility_deleted_at
           )
$$;

CREATE OR REPLACE FUNCTION secrets_v1(
    this_tenancy jsonb,
    this_visibility jsonb
)
    RETURNS SETOF secrets
    LANGUAGE sql
    STABLE PARALLEL SAFE CALLED ON NULL INPUT
AS
$$
SELECT DISTINCT ON (id) secrets.*
FROM secrets
WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, secrets)
ORDER BY id, visibility_change_set_pk DESC, visibility_deleted_at DESC NULLS FIRST
$$;


CREATE OR REPLACE FUNCTION encrypted_secret_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_definition text,
    this_description text,
    this_crypted text,
    this_version text,
    this_algorithm text,
    this_key_pair_pk ident,
    this_nonce bytea,
    this_donkey_hash bytea,
    this_created_by ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           encrypted_secrets%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO encrypted_secrets (tenancy_workspace_pk,
                                   visibility_change_set_pk,
                                   name,
                                   definition,
                                   description,
                                   crypted,
                                   version,
                                   algorithm,
                                   key_pair_pk,
                                   nonce,
                                   donkey_hash,
                                   created_by,
                                   updated_by)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_name,
            this_definition,
            this_description,
            this_crypted,
            this_version,
            this_algorithm,
            this_key_pair_pk,
            this_nonce,
            this_donkey_hash,
            this_created_by,
            this_created_by)
    RETURNING * INTO this_new_row;

    -- Purge the returning record of sensitive data to avoid accidentally
    -- deserializing these fields in application code
    this_new_row.crypted = null;
    this_new_row.version = null;
    this_new_row.algorithm = null;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
