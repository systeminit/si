CREATE TYPE tenancy_record_v1 as
(
    tenancy_workspace_pk       ident
);

CREATE OR REPLACE FUNCTION tenancy_json_to_columns_v1(this_tenancy jsonb,
                                                      OUT result tenancy_record_v1
)
AS
$$
BEGIN
    SELECT *
    FROM jsonb_to_record(this_tenancy) AS x(tenancy_workspace_pk ident)
    INTO result;
END ;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION in_tenancy_v1(
    tenancy                         jsonb,
    row_tenancy_workspace_pk        ident
)
RETURNS bool
LANGUAGE sql
IMMUTABLE
PARALLEL SAFE
AS $$
-- If any tenancy is NULL, nothing will match
SELECT (tenancy ->> 'tenancy_workspace_pk')::ident = row_tenancy_workspace_pk
$$;
