CREATE TABLE workspaces
(
    pk                          ident primary key default ident_create_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL
);
CREATE UNIQUE INDEX ON workspaces (pk);
CREATE INDEX ON workspaces (visibility_deleted_at NULLS FIRST);

CREATE OR REPLACE FUNCTION workspace_create_v1(
    this_pk ident,
    this_name text,
    OUT object json) AS
$$
DECLARE
    this_new_row           workspaces%ROWTYPE;
BEGIN

    INSERT INTO workspaces (pk, name)
    VALUES (this_pk, this_name)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION workspace_find_or_create_builtin_v1(OUT object json) AS
$$
BEGIN
    INSERT INTO workspaces (pk, name) VALUES (ident_nil_v1(), 'builtin') ON CONFLICT (pk) DO NOTHING;
    SELECT row_to_json(workspaces.*) INTO STRICT object FROM workspaces WHERE pk = ident_nil_v1();
END;
$$ LANGUAGE PLPGSQL VOLATILE;
