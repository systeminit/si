/*
    return a list of all props for all schema variants along with their internal provider ids
    in a manner suitable for constructing a tree from the list
 */
WITH RECURSIVE props_tree AS (
    SELECT
        row_to_json(p.*)  AS object,
        p.id              AS root_id,
        p.id              AS prop_id,
        p.name            AS name,
        '/'               AS path,
        ident_nil_v1()    AS parent_id,
        0::bigint         AS depth
    FROM props_v1($1, $2) AS p
    LEFT JOIN prop_belongs_to_prop_v1($1, $2) AS pbtp
        ON p.id = pbtp.object_id
    WHERE pbtp.belongs_to_id IS NULL
    UNION ALL
    SELECT
        row_to_json(child_props.*)        AS object,
        parent.root_id                    AS root_id,
        child_props.id                    AS prop_id,
        child_props.name                  AS name,
        parent.path || parent.name || '/' AS path,
        parent.prop_id                    AS parent_id,
        parent.depth + 1                  AS depth
    FROM props_v1($1, $2) AS child_props
    JOIN prop_belongs_to_prop_v1($1, $2) AS pbtp2
        ON child_props.id = pbtp2.object_id
    JOIN props_tree AS parent
        ON parent.prop_id = pbtp2.belongs_to_id
)
SELECT
    schema_variants.id AS schema_variant_id,
    props_tree.object,
    props_tree.root_id,
    props_tree.prop_id,
    props_tree.parent_id,
    props_tree.name,
    props_tree.path,
    ip.id                  AS internal_provider_id
FROM props_tree
JOIN schema_variants_v1($1, $2) schema_variants
    ON schema_variants.root_prop_id = props_tree.root_id
LEFT JOIN internal_providers_v1($1, $2) ip ON props_tree.prop_id = ip.prop_id
ORDER BY
    schema_variant_id,
    root_id,
    depth,
    name;
