CREATE OR REPLACE FUNCTION clear_workspace_v1(tenancy_to_clear jsonb)
RETURNS VOID AS
$$
DECLARE
  standard_model  standard_models%ROWTYPE;
  tenancy_record  tenancy_record_v1;
  this_table_name regclass;
BEGIN
  FOR standard_model IN SELECT * FROM standard_models
    LOOP
      this_table_name := standard_model.table_name::regclass;
      EXECUTE format(
        'DELETE FROM %1$I WHERE in_tenancy_v1(%2$L, %1$I.tenancy_workspace_pk)',
        this_table_name,
        tenancy_to_clear
      );
    END LOOP;

    EXECUTE format(
      'DELETE FROM change_sets WHERE in_tenancy_v1(%1$L, change_sets.tenancy_workspace_pk)',
      tenancy_to_clear
    );
END;
$$ LANGUAGE PLPGSQL VOLATILE;
