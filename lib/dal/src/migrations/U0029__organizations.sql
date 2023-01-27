CREATE TABLE organizations
(
    pk                          ident primary key default ident_create_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    billing_account_pk          ident                    NOT NULL
);
CREATE UNIQUE INDEX unique_organization_name_live ON organizations (
	billing_account_pk,
	name,
	(visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;
CREATE UNIQUE INDEX ON organizations (pk);
CREATE INDEX ON organizations (billing_account_pk);
CREATE INDEX ON organizations (visibility_deleted_at NULLS FIRST);

CREATE OR REPLACE FUNCTION organization_create_v1(
    this_name text,
    this_billing_account_pk ident,
    OUT object json) AS
$$
DECLARE
    this_new_row           organizations%ROWTYPE;
BEGIN
    INSERT INTO organizations (name, billing_account_pk)
    VALUES (this_name, this_billing_account_pk)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
