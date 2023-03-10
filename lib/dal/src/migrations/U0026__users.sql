CREATE TABLE users
(
    pk                          ident primary key default ident_create_v1(),
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    email                       text                     NOT NULL,
    visibility_deleted_at       timestamp with time zone
);
CREATE UNIQUE INDEX ON users (pk);
CREATE INDEX ON users (visibility_deleted_at NULLS FIRST);

CREATE TABLE user_belongs_to_workspaces
(
    pk                          ident primary key default ident_create_v1(),
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    user_pk                     ident                    NOT NULL,
    workspace_pk                ident                    NOT NULL,
    visibility_deleted_at       timestamp with time zone
);
CREATE UNIQUE INDEX ON user_belongs_to_workspaces (pk);
CREATE UNIQUE INDEX ON user_belongs_to_workspaces (user_pk, workspace_pk);
CREATE INDEX ON user_belongs_to_workspaces (user_pk);
CREATE INDEX ON user_belongs_to_workspaces (workspace_pk);
CREATE INDEX ON user_belongs_to_workspaces (visibility_deleted_at NULLS FIRST);

CREATE OR REPLACE FUNCTION user_create_v1(
    this_name text,
    this_email text,
    OUT object json) AS
$$
DECLARE
    this_new_row           users%ROWTYPE;
BEGIN
    INSERT INTO users (name, email)
    VALUES (this_name, this_email)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;


CREATE OR REPLACE FUNCTION user_associate_workspace_v1(
    this_user_pk ident,
    this_workspace_pk ident
    ) AS
$$
BEGIN
    INSERT INTO user_belongs_to_workspaces (user_pk, workspace_pk)
        VALUES (this_user_pk, this_workspace_pk)
        ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE PLPGSQL VOLATILE;
