CREATE TABLE attribute_prototypes
(
    pk                                     ident primary key default ident_create_v1(),
    id                                     ident not null default ident_create_v1(),
    tenancy_workspace_pks                  ident[],
    visibility_change_set_pk               ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at                  timestamp with time zone,
    attribute_context_prop_id              ident,
    attribute_context_internal_provider_id ident,
    attribute_context_external_provider_id ident,
    attribute_context_component_id         ident,
    created_at                             timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                             timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    func_id                                ident                   NOT NULL,
    key                                    text
);
SELECT standard_model_table_constraints_v1('attribute_prototypes');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('attribute_prototypes', 'model', 'attribute_prototype', 'Attribute Prototype');

CREATE OR REPLACE FUNCTION attribute_prototype_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_attribute_context jsonb,
    this_func_id ident,
    this_key text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record           tenancy_record_v1;
    this_visibility_record        visibility_record_v1;
    this_attribute_context_record attribute_context_record_v1;
    this_new_row                  attribute_prototypes%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);
    this_attribute_context_record := attribute_context_json_to_columns_v1(this_attribute_context);

    INSERT INTO attribute_prototypes (tenancy_workspace_pks,
                                      visibility_change_set_pk,
                                      visibility_deleted_at,
                                      attribute_context_prop_id,
                                      attribute_context_internal_provider_id,
                                      attribute_context_external_provider_id,
                                      attribute_context_component_id,
                                      func_id,
                                      key)
    VALUES (this_tenancy_record.tenancy_workspace_pks,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_attribute_context_record.attribute_context_prop_id,
            this_attribute_context_record.attribute_context_internal_provider_id,
            this_attribute_context_record.attribute_context_external_provider_id,
            this_attribute_context_record.attribute_context_component_id,
            this_func_id,
            this_key)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

-- sql functions can't take arguments of type RECORD, but they can
-- take arguments of ROWTYPE, so we need to make a specific version
-- of this function for every table we want to use it with.
CREATE OR REPLACE FUNCTION in_attribute_context_v1(
    check_context jsonb,
    record_to_check attribute_prototypes
)
    RETURNS bool
    LANGUAGE sql
    IMMUTABLE
    PARALLEL SAFE
    CALLED ON NULL INPUT
AS
$$
SELECT in_attribute_context_v1(
               check_context,
               record_to_check.attribute_context_prop_id,
               record_to_check.attribute_context_internal_provider_id,
               record_to_check.attribute_context_external_provider_id,
               record_to_check.attribute_context_component_id
           )
$$;
