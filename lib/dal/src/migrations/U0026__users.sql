CREATE TABLE users
(
    pk                          ident primary key default ident_create_v1(),
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    email                       text                     NOT NULL,
    password                    bytea                    NOT NULL,
    visibility_deleted_at       timestamp with time zone
);
CREATE UNIQUE INDEX ON users (pk);
CREATE INDEX ON users (visibility_deleted_at NULLS FIRST);

CREATE OR REPLACE FUNCTION user_create_v1(
    this_name text,
    this_email text,
    this_password bytea,
    OUT object json) AS
$$
DECLARE
    this_new_row           users%ROWTYPE;
BEGIN
    INSERT INTO users (name, email, password)
    VALUES (this_name, this_email, this_password)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
