CREATE TABLE validation_resolvers
(
    pk                                           ident primary key default ident_create_v1(),
    id                                           ident not null default ident_create_v1(),
    tenancy_workspace_pks                        ident[],
    visibility_change_set_pk                     ident                   NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at                        timestamp with time zone,
    created_at                                   timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                                   timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    validation_prototype_id                      ident                   NOT NULL,
    attribute_value_id                           ident                   NOT NULL,
    validation_func_id                           ident                   NOT NULL,
    validation_func_binding_id                   ident                   NOT NULL,
    attribute_value_func_binding_return_value_id ident                   NOT NULL
);
CREATE UNIQUE INDEX unique_validation_resolver_value_live ON validation_resolvers (
	validation_func_binding_id,
	attribute_value_id,
	tenancy_workspace_pks,
	visibility_change_set_pk,
	(visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;
SELECT standard_model_table_constraints_v1('validation_resolvers');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('validation_resolvers', 'model', 'validation_resolver', 'Validation Resolver');

CREATE OR REPLACE FUNCTION validation_resolver_create_v1(
    this_write_tenancy jsonb,
    this_read_tenancy jsonb,
    this_visibility jsonb,
    this_validation_prototype_id ident,
    this_attribute_value_id ident,
    this_func_binding_id ident,
    OUT object json) AS
$$
DECLARE
    this_write_tenancy_record                  tenancy_record_v1;
    this_visibility_record                     visibility_record_v1;
    this_new_row                               validation_resolvers%ROWTYPE;
    this_func_id                               ident;
    this_attr_val_func_binding_return_value_id ident;
BEGIN
    this_write_tenancy_record := tenancy_json_to_columns_v1(this_write_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    SELECT DISTINCT ON (id) func_binding_return_value_id
    INTO STRICT this_attr_val_func_binding_return_value_id
    FROM attribute_values
    WHERE in_tenancy_and_visible_v1(this_read_tenancy, this_visibility, attribute_values)
      AND id = this_attribute_value_id
    ORDER BY id,
             visibility_change_set_pk DESC,
             visibility_deleted_at DESC NULLS FIRST;

    SELECT DISTINCT ON (object_id) belongs_to_id
    INTO STRICT this_func_id
    FROM func_binding_belongs_to_func
    WHERE in_tenancy_and_visible_v1(this_read_tenancy, this_visibility, func_binding_belongs_to_func)
      AND object_id = this_func_binding_id
    ORDER BY object_id DESC,
             belongs_to_id DESC,
             visibility_change_set_pk DESC,
             visibility_deleted_at DESC NULLS FIRST;

    INSERT INTO validation_resolvers (tenancy_workspace_pks,
                                      visibility_change_set_pk,
                                      visibility_deleted_at,
                                      validation_prototype_id,
                                      attribute_value_id,
                                      validation_func_id,
                                      validation_func_binding_id,
                                      attribute_value_func_binding_return_value_id)
    VALUES (this_write_tenancy_record.tenancy_workspace_pks,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_validation_prototype_id,
            this_attribute_value_id,
            this_func_id,
            this_func_binding_id,
            this_attr_val_func_binding_return_value_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
