CREATE TABLE billing_accounts
(
    pk                          ident primary key default ident_create_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    description                 text
);
CREATE UNIQUE INDEX unique_billing_account_name_live ON billing_accounts (
	name,
	(visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;
CREATE UNIQUE INDEX ON billing_accounts (pk);
CREATE INDEX ON billing_accounts (visibility_deleted_at NULLS FIRST);

CREATE OR REPLACE FUNCTION billing_account_create_v1(
    this_name text,
    this_description text,
    OUT object json) AS
$$
DECLARE
    this_new_row           billing_accounts%ROWTYPE;
BEGIN
    INSERT INTO billing_accounts (name, description)
    VALUES (this_name, this_description)
    RETURNING * INTO this_new_row;
    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
