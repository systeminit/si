--- we no longer use a sha256 hash on a function's code, blake3 is used instead
ALTER TABLE func_bindings
    ADD COLUMN func_id     ident,
    ADD COLUMN code_blake3 TEXT,
    DROP COLUMN code_sha256;

DROP FUNCTION func_binding_create_v1;

CREATE OR REPLACE FUNCTION func_binding_create_v2(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_args json,
    this_func_id ident,
    this_backend_kind text,
    this_code_blake3 text,
    OUT object json) AS
$$
DECLARE
    this_tenancy_record    tenancy_record_v1;
    this_visibility_record visibility_record_v1;
    this_new_row           func_bindings%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);
    this_visibility_record := visibility_json_to_columns_v1(this_visibility);

    INSERT INTO func_bindings (tenancy_workspace_pk,
                               visibility_change_set_pk,
                               args,
                               backend_kind,
                               code_blake3,
                               func_id)
    VALUES (this_tenancy_record.tenancy_workspace_pk,
            this_visibility_record.visibility_change_set_pk,
            this_args,
            this_backend_kind,
            COALESCE(this_code_blake3, '0'),
            this_func_id)
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

