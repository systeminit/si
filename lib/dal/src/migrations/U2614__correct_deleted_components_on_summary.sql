-- This exists to fix bad summaries for deleted components pre migration U2613

-- Created and deleted on the same changeset
-- if there is a component with deleted not null and a fully matching (id, changeset, workspace) summary,
-- update summary with deletion data
UPDATE summary_diagram_components sc SET
    visibility_deleted_at = c.visibility_deleted_at,
    change_status = 'deleted',
    deleted_info = jsonb_build_object(
         'actor', jsonb_build_object('kind', 'system', 'label', 'migration 2614'),
         'timestamp', now()
                )
FROM components c
WHERE c.visibility_change_set_pk = sc.visibility_change_set_pk
  AND c.tenancy_workspace_pk = sc.tenancy_workspace_pk
  AND c.tenancy_workspace_pk != ident_nil_v1()
  AND c.id = sc.component_id
  AND c.visibility_deleted_at IS NOT NULL;

-- Things got merged and deleted on changeset
-- There's no summary entry on the changeset the component got deleted, so we insert it
INSERT INTO summary_diagram_components (id, tenancy_workspace_pk, visibility_change_set_pk,
                                        visibility_deleted_at, created_at, component_id,
                                        display_name, node_id, schema_name,
                                        schema_id, schema_variant_id,
                                        schema_variant_name, schema_category, position, size, color, node_type,
                                        change_status, has_resource, created_info, updated_info, deleted_info,
                                        sockets, parent_node_id, child_node_ids)
SELECT
    head_sc.id, head_sc.tenancy_workspace_pk, c.visibility_change_set_pk,
    c.visibility_deleted_at, head_sc.created_at, head_sc.component_id,
    head_sc.display_name, head_sc.node_id, head_sc.schema_name,
    head_sc.schema_id, head_sc.schema_variant_id,
    head_sc.schema_variant_name, head_sc.schema_category, head_sc.position, head_sc.size, head_sc.color, head_sc.node_type,
    'deleted', head_sc.has_resource, head_sc.created_info, head_sc.updated_info,
    jsonb_build_object( -- deleted info
            'actor', jsonb_build_object('kind', 'system', 'label', 'migration 2614'),
            'timestamp', now()
    ),
    head_sc.sockets, head_sc.parent_node_id, head_sc.child_node_ids
FROM components c
    -- This + `filter_sc.id IS NULL` at the end is an outer left join, which means
    -- only get components that dont have a fully matching summaries
    LEFT JOIN summary_diagram_components filter_sc ON
        c.id = filter_sc.component_id AND
        c.visibility_change_set_pk = filter_sc.visibility_change_set_pk AND
        c.tenancy_workspace_pk = filter_sc.tenancy_workspace_pk
    -- get matching head summaries (which always exist in this case)
    JOIN summary_diagram_components head_sc ON
        c.id = head_sc.component_id AND
        head_sc.visibility_change_set_pk = ident_nil_v1() AND
        c.tenancy_workspace_pk = head_sc.tenancy_workspace_pk
    -- Do this for deleted components with no head version
    WHERE c.visibility_deleted_at IS NOT NULL
      AND c.visibility_change_set_pk != ident_nil_v1()
      AND c.tenancy_workspace_pk != ident_nil_v1()
      AND filter_sc.id IS NULL;


