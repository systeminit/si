CREATE TABLE key_pairs
(
    pk                          ident primary key default ident_create_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    workspace_pk                ident                    NOT NULL,
    created_lamport_clock       bigserial                NOT NULL,
    public_key                  text                     NOT NULL,
    secret_key_crypted          text                     NOT NULL,
    secret_key_nonce            text                     NOT NULL,
    secret_key_key_hash         text                     NOT NULL
);
CREATE UNIQUE INDEX ON key_pairs (pk);
CREATE INDEX ON key_pairs (visibility_deleted_at NULLS FIRST);

CREATE OR REPLACE FUNCTION key_pair_create_v1(
    this_name text,
    this_workspace_pk ident,
    this_public_key text,
    this_secret_key_crypted text,
    this_secret_key_nonce text,
    this_secret_key_key_hash text,
    OUT object json) AS
$$
DECLARE
    this_new_row           key_pairs%ROWTYPE;
BEGIN
    INSERT INTO key_pairs (name,
                           workspace_pk,
                           public_key,
                           secret_key_crypted,
                           secret_key_nonce,
                           secret_key_key_hash)
    VALUES (this_name,
            this_workspace_pk,
            this_public_key,
            this_secret_key_crypted,
            this_secret_key_nonce,
            this_secret_key_key_hash)
    RETURNING * INTO this_new_row;
    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
