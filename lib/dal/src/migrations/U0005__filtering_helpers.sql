-- All standard model tables should automatically have versions of this where
-- the type of `reference` is that table's `ROWTYPE`. This only exists as a
-- fallback, in case a table wasn't set up through the "normal" standard
-- model means.
CREATE OR REPLACE FUNCTION in_tenancy_and_visible_v1(read_tenancy jsonb,
                                                     check_visibility jsonb,
                                                     reference record,
                                                     OUT result bool
)
AS
$$
BEGIN
    result :=
                in_tenancy_v1(read_tenancy,
                              reference.tenancy_universal,
                              reference.tenancy_billing_account_ids,
                              reference.tenancy_organization_ids,
                              reference.tenancy_workspace_ids)
                AND is_visible_v1(
                        check_visibility,
                        reference.visibility_change_set_pk,
                        reference.visibility_deleted_at);
END ;
$$ LANGUAGE PLPGSQL IMMUTABLE;
