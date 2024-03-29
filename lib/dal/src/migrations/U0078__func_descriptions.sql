CREATE TABLE func_descriptions
(
    pk                          ident primary key                 default ident_create_v1(),
    id                          ident                    not null default ident_create_v1(),
    tenancy_workspace_pk        ident,
    visibility_change_set_pk    ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at       timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),

    func_id                     ident                    NOT NULL,
    schema_variant_id           ident                    NOT NULL,
    serialized_contents         jsonb                    NOT NULL,
    response_type               text                     NOT NULL
);

CREATE UNIQUE INDEX unique_func_descriptions
    ON func_descriptions (func_id,
                          schema_variant_id,
                          tenancy_workspace_pk,
                          visibility_change_set_pk);

SELECT standard_model_table_constraints_v1('func_descriptions');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('func_descriptions', 'model', 'func_description', 'Func Description');

CREATE OR REPLACE FUNCTION func_description_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_func_id ident,
    this_schema_variant_id ident,
    this_serialized_contents jsonb,
    this_response_type text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           func_descriptions%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO func_descriptions (tenancy_workspace_pk,
                                   visibility_change_set_pk,
                                   func_id,
                                   schema_variant_id,
                                   serialized_contents,
                                   response_type)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_func_id,
            this_schema_variant_id,
            this_serialized_contents,
            this_response_type)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
