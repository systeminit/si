CREATE TABLE workspaces
(
    pk                          ident primary key default ident_create_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    organization_pk             ident                    NOT NULL
);
CREATE UNIQUE INDEX unique_workspaces_name_live ON workspaces (
	organization_pk,
	name,
	(visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;
CREATE UNIQUE INDEX ON workspaces (pk);
CREATE INDEX ON workspaces (organization_pk);
CREATE INDEX ON workspaces (visibility_deleted_at NULLS FIRST);

CREATE OR REPLACE FUNCTION workspace_create_v1(
    this_name text,
    this_organization_pk ident,
    OUT object json) AS
$$
DECLARE
    this_new_row           workspaces%ROWTYPE;
BEGIN

    INSERT INTO workspaces (name, organization_pk)
    VALUES (this_name, this_organization_pk)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
