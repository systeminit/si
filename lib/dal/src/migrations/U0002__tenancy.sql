CREATE TYPE tenancy_record_v1 as
(
    tenancy_universal           bool,
    tenancy_billing_account_ids bigint[],
    tenancy_organization_ids    bigint[],
    tenancy_workspace_ids       bigint[]
);

CREATE OR REPLACE FUNCTION tenancy_json_to_columns_v1(this_tenancy jsonb,
                                                      OUT result tenancy_record_v1
)
AS
$$
BEGIN
    SELECT *
    FROM jsonb_to_record(this_tenancy) AS x(
                                            tenancy_universal bool,
                                            tenancy_billing_account_ids bigint[],
                                            tenancy_organization_ids bigint[],
                                            tenancy_workspace_ids bigint[]
        )
    INTO result;
END ;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION in_tenancy_v1(check_tenancy jsonb,
                                         this_tenancy_universal bool,
                                         this_tenancy_billing_account_ids bigint[],
                                         this_tenancy_organization_ids bigint[],
                                         this_tenancy_workspace_ids bigint[],
                                         OUT result bool
)
AS
$$
DECLARE
    check_tenancy_record  tenancy_record_v1;
    empty_check           bool;
    universal_check       bool;
    billing_account_check bool;
    organization_check bool;
    workspace_check bool;
    empty_billing_account_list bool;
    empty_organization_list bool;
    empty_workspace_list bool;
BEGIN
    check_tenancy_record := tenancy_json_to_columns_v1(check_tenancy);
    RAISE DEBUG 'in_tenancy: % vs: u:% b:% o:% w:%', check_tenancy, this_tenancy_universal, this_tenancy_billing_account_ids, this_tenancy_organization_ids, this_tenancy_workspace_ids;

    empty_billing_account_list := cardinality(check_tenancy_record.tenancy_billing_account_ids) = 0;
    RAISE DEBUG 'empty_billing_account_list: %', empty_billing_account_list;
    empty_organization_list := cardinality(check_tenancy_record.tenancy_organization_ids) = 0;
    RAISE DEBUG 'empty_organization_list: %', empty_organization_list;
    empty_workspace_list := cardinality(check_tenancy_record.tenancy_workspace_ids) = 0;
    RAISE DEBUG 'empty_workspace_list: %', empty_workspace_list;

    empty_check := NOT (check_tenancy_record.tenancy_universal = false
        AND empty_billing_account_list
        AND empty_organization_list
        AND empty_workspace_list);
    RAISE DEBUG 'empty_check: %', empty_check;

    universal_check := (check_tenancy_record.tenancy_universal AND check_tenancy_record.tenancy_universal = this_tenancy_universal);
    RAISE DEBUG 'universal_check: %', universal_check;

    billing_account_check := (NOT empty_billing_account_list AND check_tenancy_record.tenancy_billing_account_ids <@ this_tenancy_billing_account_ids);
    RAISE DEBUG 'billing_account_check: %', billing_account_check;

    organization_check := (NOT empty_organization_list AND check_tenancy_record.tenancy_organization_ids <@ this_tenancy_organization_ids);
    RAISE DEBUG 'organization_check: %', organization_check;

    workspace_check := (NOT empty_workspace_list AND check_tenancy_record.tenancy_workspace_ids <@ this_tenancy_workspace_ids);
    RAISE DEBUG 'workspace_check: %', workspace_check;

    result := (empty_check
        AND (
                       universal_check
                       OR
                       billing_account_check
                       OR
                       organization_check
                       OR
                       workspace_check
                   )
        );
    RAISE DEBUG 'tenancy check result: %', result;
END ;
$$ LANGUAGE PLPGSQL IMMUTABLE;
