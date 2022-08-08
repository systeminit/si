/*
    This query groups arguments that belong to an attribute prototype by name. For every argument that shares the same
    name, they will be in the same "array".

    { key: name, value: [argument_with_same_name_1, argument_with_same_name_2] },
    { key: name, value: [argument_that_only_has_this_name] }
 */

SELECT name, array_agg(arguments) as arguments
FROM (SELECT DISTINCT ON (attribute_prototype_arguments.id) attribute_prototype_arguments.id,
                                                            attribute_prototype_arguments.visibility_change_set_pk,
                                                            attribute_prototype_arguments.visibility_deleted_at,
                                                            attribute_prototype_arguments.name           AS name,
                                                            row_to_json(attribute_prototype_arguments.*) AS arguments
      FROM attribute_prototype_arguments
      WHERE in_tenancy_v1($1, attribute_prototype_arguments.tenancy_universal,
                          attribute_prototype_arguments.tenancy_billing_account_ids,
                          attribute_prototype_arguments.tenancy_organization_ids,
                          attribute_prototype_arguments.tenancy_workspace_ids)
        AND is_visible_v1($2, attribute_prototype_arguments.visibility_change_set_pk,
                          attribute_prototype_arguments.visibility_deleted_at)
        AND attribute_prototype_arguments.attribute_prototype_id = $3

      ORDER BY attribute_prototype_arguments.id,
               visibility_change_set_pk DESC,
               visibility_deleted_at DESC NULLS FIRST) as apa_found
GROUP BY name;
