CREATE TABLE action_prototypes
(
    pk                       ident primary key                 DEFAULT ident_create_v1(),
    id                       ident                    NOT NULL DEFAULT ident_create_v1(),
    tenancy_workspace_pk     ident,
    visibility_change_set_pk ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at    timestamp with time zone,
    created_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at               timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    func_id                  ident                    NOT NULL,
    kind                     text                     NOT NULL,
    schema_variant_id        ident                    NOT NULL
);

SELECT standard_model_table_constraints_v1('action_prototypes');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('action_prototypes', 'model', 'action_prototype', 'Action Prototype');

ALTER TABLE action_prototypes
    ADD CONSTRAINT valid_kind_check CHECK (kind IN ('create', 'delete', 'other', 'refresh'));

CREATE OR REPLACE FUNCTION action_prototype_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_func_id ident,
    this_kind text,
    this_schema_variant_id ident,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           action_prototypes%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO action_prototypes (tenancy_workspace_pk,
                                   visibility_change_set_pk,
                                   func_id,
                                   kind,
                                   schema_variant_id)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_func_id,
            this_kind,
            this_schema_variant_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;