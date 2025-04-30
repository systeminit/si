CREATE TABLE user_invited_to_workspace
(
    pk                          ident primary key default ident_create_v1(),
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    email                       text                     NOT NULL,
    workspace_pk                ident                    NOT NULL,
    visibility_deleted_at       timestamp with time zone
);
CREATE UNIQUE INDEX ON user_invited_to_workspace (pk);
CREATE UNIQUE INDEX ON user_invited_to_workspace (email, workspace_pk);
CREATE INDEX ON user_invited_to_workspace (email);
CREATE INDEX ON user_invited_to_workspace (workspace_pk);
CREATE INDEX ON user_invited_to_workspace (visibility_deleted_at NULLS FIRST);

CREATE OR REPLACE FUNCTION user_invite_to_workspace_v1(
    this_email text,
    this_workspace_pk ident
) RETURNS void AS
$$
BEGIN
    INSERT INTO user_invited_to_workspace (email, workspace_pk)
        VALUES (this_email, this_workspace_pk)
        ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION user_associate_workspace_invites_v1(
    this_user_pk ident,
    this_user_email text
) RETURNS void AS
$$
DECLARE
    workspace_pks ident[];
    pk ident;
BEGIN
    WITH pks AS (
        DELETE FROM user_invited_to_workspace
        WHERE email = this_user_email
        RETURNING workspace_pk
    )
    SELECT array_agg(workspace_pk)
    FROM pks
    INTO workspace_pks;

    FOREACH pk IN ARRAY coalesce(workspace_pks, '{}')
        LOOP
		    EXECUTE user_associate_workspace_v1(this_user_pk, pk);
        END LOOP;
END;
$$ LANGUAGE PLPGSQL VOLATILE;
