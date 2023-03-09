CREATE TABLE workspaces
(
    pk                          ident primary key default ident_create_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    billing_account_pk          ident                    NOT NULL
);
CREATE UNIQUE INDEX unique_workspaces_name_live ON workspaces (
	billing_account_pk,
	name,
	(visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;
CREATE UNIQUE INDEX ON workspaces (pk);
CREATE INDEX ON workspaces (billing_account_pk);
CREATE INDEX ON workspaces (visibility_deleted_at NULLS FIRST);

CREATE OR REPLACE FUNCTION workspace_create_v1(
    this_name text,
    this_billing_account_pk ident,
    OUT object json) AS
$$
DECLARE
    this_new_row           workspaces%ROWTYPE;
BEGIN

    INSERT INTO workspaces (name, billing_account_pk)
    VALUES (this_name, this_billing_account_pk)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION workspace_find_or_create_builtin_v1(OUT object json) AS
$$
BEGIN
    INSERT INTO billing_accounts (pk, name) VALUES (ident_nil_v1(), 'builtin') ON CONFLICT (pk) DO NOTHING;
    INSERT INTO workspaces (pk, name, billing_account_pk) VALUES (ident_nil_v1(), 'builtin', ident_nil_v1()) ON CONFLICT (pk) DO NOTHING;
    SELECT row_to_json(workspaces.*) INTO STRICT object FROM workspaces WHERE pk = ident_nil_v1();
END;
$$ LANGUAGE PLPGSQL VOLATILE;
