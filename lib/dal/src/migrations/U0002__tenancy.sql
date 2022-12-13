CREATE TYPE tenancy_record_v1 as
(
    tenancy_universal           bool,
    tenancy_billing_account_ids ident[],
    tenancy_organization_ids    ident[],
    tenancy_workspace_ids       ident[]
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
                                            tenancy_billing_account_ids ident[],
                                            tenancy_organization_ids ident[],
                                            tenancy_workspace_ids ident[]
        )
    INTO result;
END ;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION in_tenancy_v1(
    read_tenancy                    jsonb,
    row_tenancy_universal           bool,
    row_tenancy_billing_account_ids ident[],
    row_tenancy_organization_ids    ident[],
    row_tenancy_workspace_ids       ident[]
)
RETURNS bool
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
SELECT
    (row_tenancy_universal AND row_tenancy_universal = (read_tenancy -> 'tenancy_universal')::bool)
    -- Unfortunately jsonb only has an easy way to say "are any elements of text[] in the jsonb array", but not doing
    -- the same for a ident[], so we translate the jsonb array into a PG array, and use ARRAY && ARRAY for the check.
    OR (translate(read_tenancy ->> 'tenancy_billing_account_ids', '[]', '{}')::ident[] && row_tenancy_billing_account_ids)
    OR (translate(read_tenancy ->> 'tenancy_organization_ids', '[]', '{}')::ident[] && row_tenancy_organization_ids)
    OR (translate(read_tenancy ->> 'tenancy_workspace_ids', '[]', '{}')::ident[] && row_tenancy_workspace_ids)
$$;
