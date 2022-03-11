CREATE TABLE func_executions
(
    pk                           bigserial                PRIMARY KEY,
    state                        text                     NOT NULL,
    func_id                      bigint                   NOT NULL,
    func_binding_id              bigint                   NOT NULL,
    func_binding_args            jsonb                    NOT NULL,
    backend_kind                 text                     NOT NULL,
    backend_response_type        text                     NOT NULL,
    func_binding_return_value_id bigint,
    handler                      text,
    code_base64                  text,
    unprocessed_value            jsonb,
    value                        jsonb,
    output_stream                jsonb,
    function_failure             jsonb,
    tenancy_universal            bool,
    tenancy_billing_account_ids  bigint[],
    tenancy_organization_ids     bigint[],
    tenancy_workspace_ids        bigint[],
    created_at                   timestamp with time zone NOT NULL DEFAULT NOW(),
    updated_at                   timestamp with time zone NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION func_execution_create_v1(
    this_tenancy                      jsonb,
    this_state                        text,
    this_func_id                      bigint,
    this_func_binding_id              bigint,
    this_func_binding_args            jsonb,
    this_backend_kind                 text,
    this_backend_response_type        text,
    this_handler                      text,
    this_code_base64                  text,
  OUT object json) AS
$$
DECLARE
    this_tenancy_record tenancy_record_v1;
    this_new_row        func_executions%ROWTYPE;
BEGIN
    this_tenancy_record := tenancy_json_to_columns_v1(this_tenancy);

    INSERT INTO func_executions (
      tenancy_universal,
      tenancy_billing_account_ids,
      tenancy_organization_ids,
      tenancy_workspace_ids,
      state,
      func_id,
      func_binding_id,
      func_binding_args,
      backend_kind,
      backend_response_type,
      handler,
      code_base64
    )
    VALUES (
      this_tenancy_record.tenancy_universal,
      this_tenancy_record.tenancy_billing_account_ids,
      this_tenancy_record.tenancy_organization_ids,
      this_tenancy_record.tenancy_workspace_ids,
      this_state,
      this_func_id,
      this_func_binding_id,
      this_func_binding_args,
      this_backend_kind,
      this_backend_response_type,
      this_handler,
      this_code_base64
    )
    RETURNING * INTO this_new_row;

    object := row_to_json(this_new_row);
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION func_execution_set_state_v1(
    this_pk bigint,
    this_state text,
  OUT object json) AS
$$
BEGIN
  UPDATE func_executions
    SET state = this_state, updated_at = now()
    WHERE pk = this_pk
    RETURNING row_to_json(func_executions.*)
    INTO object;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION func_execution_set_output_stream_v1(
    this_pk bigint,
    this_output_stream jsonb,
  OUT object json) AS
$$
BEGIN
  UPDATE func_executions
    SET output_stream = this_output_stream, updated_at = now()
    WHERE pk = this_pk
    RETURNING row_to_json(func_executions.*)
    INTO object;
END;
$$ LANGUAGE PLPGSQL VOLATILE;

CREATE OR REPLACE FUNCTION func_execution_set_return_value_v1(
    this_pk bigint,
    this_func_binding_return_value_id bigint,
    this_value jsonb,
    this_unprocessed_value jsonb,
  OUT object json) AS
$$
BEGIN
  UPDATE func_executions
    SET func_binding_return_value_id = this_func_binding_return_value_id,
        value = this_value,
        unprocessed_value = this_unprocessed_value,
        updated_at = now()
    WHERE pk = this_pk
    RETURNING row_to_json(func_executions.*)
    INTO object;
END;
$$ LANGUAGE PLPGSQL VOLATILE;
