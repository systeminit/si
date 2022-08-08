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

CREATE OR REPLACE FUNCTION in_tenancy_v1(read_tenancy jsonb,
                                         row_tenancy_universal bool,
                                         row_tenancy_billing_account_ids bigint[],
                                         row_tenancy_organization_ids bigint[],
                                         row_tenancy_workspace_ids bigint[],
                                         OUT result bool
)
AS
$$
DECLARE
    read_tenancy_record   tenancy_record_v1;
    universal_check       bool;
    billing_account_check bool;
    organization_check    bool;
    workspace_check       bool;
BEGIN
    read_tenancy_record := tenancy_json_to_columns_v1(read_tenancy);
    RAISE DEBUG 'in_tenancy: % vs: u:% b:% o:% w:%', read_tenancy, row_tenancy_universal, row_tenancy_billing_account_ids, row_tenancy_organization_ids, row_tenancy_workspace_ids;

    universal_check := row_tenancy_universal AND row_tenancy_universal = read_tenancy_record.tenancy_universal;
    RAISE DEBUG 'universal_check: %', universal_check;

    billing_account_check := read_tenancy_record.tenancy_billing_account_ids && row_tenancy_billing_account_ids;
    RAISE DEBUG 'billing_account_check: %', billing_account_check;

    organization_check := read_tenancy_record.tenancy_organization_ids && row_tenancy_organization_ids;
    RAISE DEBUG 'organization_check: %', organization_check;

    workspace_check := read_tenancy_record.tenancy_workspace_ids && row_tenancy_workspace_ids;
    RAISE DEBUG 'workspace_check: %', workspace_check;

    result := (universal_check OR billing_account_check OR organization_check OR workspace_check);
    RAISE DEBUG 'tenancy check result: %', result;
END ;
$$ LANGUAGE PLPGSQL IMMUTABLE;
