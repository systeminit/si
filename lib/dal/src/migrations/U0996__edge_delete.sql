CREATE OR REPLACE FUNCTION edge_deletion_v1(
    this_tenancy jsonb,
    this_visibility jsonb,
    this_edge_id ident,
    this_user_id ident
)
    RETURNS TABLE
            (
                object json
            )
AS
$$
DECLARE
    deleted_timestamp    timestamp with time zone;
BEGIN

    SELECT delete_by_id_v1('edges', this_tenancy, this_visibility, this_edge_id) INTO deleted_timestamp;

    -- Ensure we now set the actor of who has deleted the component
    PERFORM update_by_id_v1('edges',
            'deletion_user_id',
            this_tenancy,
            this_visibility || jsonb_build_object('visibility_deleted_at', deleted_timestamp),
            this_edge_id,
            this_user_id);
END;
$$ LANGUAGE PLPGSQL STABLE;
