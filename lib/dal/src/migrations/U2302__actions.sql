CREATE TABLE actions
(
    pk                          ident primary key default ident_create_v1(),
    id                          ident not null default ident_create_v1(),
    tenancy_workspace_pk        ident,
    visibility_change_set_pk    ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    index                       smallint                 NOT NULL,
    action_prototype_id         ident                    NOT NULL,
    change_set_pk               ident                    NOT NULL,
    component_id                ident                    NOT NULL
);
SELECT standard_model_table_constraints_v1('actions');

-- TODO: remove this unique check, it only exists to avoid multiple of the same action messing sorting
-- but we should fix this eventually properly
CREATE UNIQUE INDEX actions_unique ON actions (tenancy_workspace_pk, visibility_change_set_pk, action_prototype_id, change_set_pk, component_id) WHERE visibility_deleted_at IS NULL;

ALTER TABLE action_prototypes ADD COLUMN name TEXT;

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('actions', 'model', 'action', 'Action');

CREATE OR REPLACE FUNCTION action_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_action_prototype_id ident,
    this_component_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           actions%ROWTYPE;
    this_index             smallint;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    SELECT COUNT(id)
    FROM actions
    WHERE change_set_pk = this_visibility_record.visibility_change_set_pk
    INTO STRICT this_index;

    INSERT INTO actions (tenancy_workspace_pk,
                         visibility_change_set_pk,
						 index,
                         action_prototype_id,
                         change_set_pk,
                         component_id)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
		    this_index,
            this_action_prototype_id,
            this_visibility_record.visibility_change_set_pk,
            this_component_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
