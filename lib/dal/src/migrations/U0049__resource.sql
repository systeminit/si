CREATE TABLE resources
(
    pk                          bigserial PRIMARY KEY,
    id                          bigserial                NOT NULL,
    tenancy_universal           bool                     NOT NULL,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[],
    visibility_change_set_pk    bigint                   NOT NULL DEFAULT -1,
    visibility_deleted_at       timestamp with time zone,
    data                        jsonb                    NOT NULL,
    component_id                bigint                   NOT NULL,
    system_id                   bigint                   NOT NULL,
    created_at                  timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                  timestamp with time zone NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX unique_resources
    ON resources (component_id,
                  system_id,
                  visibility_change_set_pk,
                  (visibility_deleted_at IS NULL))
    WHERE visibility_deleted_at IS NULL;

SELECT standard_model_table_constraints_v1('resources');
INSERT INTO standard_models (table_name, table_type, history_event_label_base, history_event_message_name)
VALUES ('resources', 'model', 'resource', 'Resource');

CREATE OR REPLACE FUNCTION resource_create_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_data jsonb,
    this_component_id bigint,
    this_system_id bigint,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           resources%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO resources (tenancy_universal,
                           tenancy_billing_account_ids,
                           tenancy_organization_ids,
                           tenancy_workspace_ids,
                           visibility_change_set_pk,
                           visibility_deleted_at,
                           data,
                           component_id,
                           system_id)
    VALUES (this_tenancy_record.tenancy_universal,
            this_tenancy_record.tenancy_billing_account_ids,
            this_tenancy_record.tenancy_organization_ids,
            this_tenancy_record.tenancy_workspace_ids,
            this_visibility_record.visibility_change_set_pk,
            this_visibility_record.visibility_deleted_at,
            this_data,
            this_component_id,
            this_system_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;
