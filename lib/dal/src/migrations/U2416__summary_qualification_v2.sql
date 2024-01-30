CREATE OR REPLACE FUNCTION summary_qualification_update_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_component_id ident,
    this_component_name text,
    this_total bigint,
    this_warned bigint,
    this_succeeded bigint,
    this_failed bigint,
    this_deleted_at timestamp with time zone,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           summary_qualifications%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO summary_qualifications
    (id, component_id, component_name, tenancy_workspace_pk, visibility_change_set_pk, total, warned, succeeded, failed,
     visibility_deleted_at)
    VALUES (this_component_id, this_component_id, this_component_name, this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_total, this_warned, this_succeeded, this_failed, this_deleted_at)
    ON CONFLICT (id, tenancy_workspace_pk, visibility_change_set_pk)
        DO UPDATE SET component_name        = this_component_name,
                      total                 = this_total,
                      warned                = this_warned,
                      succeeded             = this_succeeded,
                      failed                = this_failed,
                      visibility_deleted_at = this_deleted_at
    RETURNING * INTO this_new_row;
END
$$ LANGUAGE PLPGSQL VOLATILE;
