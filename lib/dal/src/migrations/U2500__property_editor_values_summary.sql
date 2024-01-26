CREATE TABLE property_editor_values_summaries (
    pk                        ident PRIMARY KEY                 DEFAULT ident_create_v1(),
    id                        ident                    NOT NULL,
    tenancy_workspace_pk      ident,
    visibility_change_set_pk  ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at     TIMESTAMP WITH TIME ZONE,
    created_at                TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT clock_timestamp(),
    updated_at                TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT clock_timestamp(),
    property_editor_values    JSONB                    NOT NULL
);

SELECT standard_model_table_constraints_v1('property_editor_values_summaries');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
    VALUES ('property_editor_values_summaries', 'model', 'property_editor_values', 'Property Editor Values Summary');

CREATE OR REPLACE FUNCTION property_editor_values_summary_create_or_update_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_summary jsonb,
    OUT object json
) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           property_editor_values_summaries%ROWTYPE;
    this_existing_row      record;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    SELECT * INTO this_existing_row FROM get_by_id_v1('property_editor_values_summaries', this_tenancy, this_visibility, this_component_id);
    IF this_existing_row.id IS NULL THEN
        INSERT INTO property_editor_values_summaries (
            id,
            tenancy_workspace_pk,
            visibility_change_set_pk,
            visibility_deleted_at,
            property_editor_values
        )
        VALUES (
            this_component_id,
            this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_summary
        )
        RETURNING * INTO this_new_row;

        object = row_to_json(this_new_row);
    ELSE
        PERFORM update_by_id_v1(
            'property_editor_values_summaries',
            'property_editor_values',
            this_tenancy,
            this_visibility,
            this_component_id,
            this_summary
        );
        -- update_by_id_v1 only returns the updated_at, not the full record.
        SELECT get.object INTO object
        FROM get_by_id_v1(
            'property_editor_values_summaries',
            this_tenancy,
            this_visibility,
            this_component_id
        ) AS get;
    END IF;
END
$$ LANGUAGE PLPGSQL VOLATILE;
