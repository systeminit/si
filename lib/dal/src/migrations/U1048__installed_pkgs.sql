CREATE TABLE installed_pkgs 
(
    pk                          ident                    PRIMARY KEY DEFAULT ident_create_v1(),
    id                          ident                    NOT NULL DEFAULT ident_create_v1(),
    tenancy_workspace_pk        ident,
    visibility_change_set_pk    ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    name                        text                     NOT NULL,
    root_hash                   text                     NOT NULL
);
CREATE UNIQUE INDEX unique_pkg_hash ON installed_pkgs (
	root_hash,
	tenancy_workspace_pk,
	visibility_change_set_pk,
	(visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;
SELECT standard_model_table_constraints_v1('installed_pkgs');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('installed_pkgs', 'model', 'installed_pkg', 'Installed Package');

CREATE OR REPLACE FUNCTION installed_pkg_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_name text,
    this_root_hash text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           installed_pkgs%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO installed_pkgs (
        tenancy_workspace_pk, visibility_change_set_pk, visibility_deleted_at,
        name, root_hash
    ) VALUES (
        this_tenancy_record.tenancy_workspace_pk,
        this_visibility_record.visibility_change_set_pk,
        this_visibility_record.visibility_deleted_at, this_name, this_root_hash 
    )
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
