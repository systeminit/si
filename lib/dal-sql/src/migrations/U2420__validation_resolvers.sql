CREATE TABLE validation_resolvers
(
    pk                                           ident primary key default ident_create_v1(),
    id                                           ident not null default ident_create_v1(),
    tenancy_workspace_pk                         ident,
    visibility_change_set_pk                     ident                    NOT NULL DEFAULT ident_nil_v1(),
    visibility_deleted_at                        timestamp with time zone,
    created_at                                   timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at                                   timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    prop_id                                      ident                    NOT NULL,
    component_id                                 ident                    NOT NULL,
    value                                        jsonb                    NOT NULL
);
CREATE UNIQUE INDEX unique_validation_resolver_value_live ON validation_resolvers (
    prop_id,
    component_id,
	tenancy_workspace_pk,
	visibility_change_set_pk);
SELECT standard_model_table_constraints_v1('validation_resolvers');

INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('validation_resolvers', 'model', 'validation_resolver', 'Validation Resolver');

CREATE OR REPLACE FUNCTION validation_resolver_upsert_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_prop_id ident,
    this_component_id ident,
    this_value jsonb,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record                        tenancy_record_v1;
    this_visibility_record                     visibility_record_v1;
    this_new_row                               validation_resolvers%ROWTYPE;
    this_existing_id                           ident;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    SELECT id FROM validation_resolvers_v1(this_tenancy, this_visibility)
    INTO this_existing_id
    WHERE prop_id = this_prop_id AND component_id = this_component_id;

    IF this_existing_id IS NOT NULL THEN
		PERFORM update_by_id_v1('validation_resolvers', 'value', this_tenancy, this_visibility, this_existing_id, this_value);

        SELECT * FROM validation_resolvers_v1(this_tenancy, this_visibility)
        INTO this_new_row
        WHERE prop_id = this_prop_id AND component_id = this_component_id;
    ELSE
        INSERT INTO validation_resolvers (tenancy_workspace_pk,
                                          visibility_change_set_pk,
									      prop_id,
									      component_id,
									      value)
        VALUES (this_tenancy_record.tenancy_workspace_pk,
                this_visibility_record.visibility_change_set_pk,
			    this_prop_id,
			    this_component_id,
			    this_value)
        RETURNING * INTO this_new_row;
    END IF;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
