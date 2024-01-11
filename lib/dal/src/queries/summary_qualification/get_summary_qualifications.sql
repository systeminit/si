SELECT row_to_json(a) AS object
FROM (SELECT DISTINCT ON (components.id) components.id AS component_id,
                                         summary_qualifications.component_name,
                                         summary_qualifications.total,
                                         summary_qualifications.warned,
                                         summary_qualifications.succeeded,
                                         summary_qualifications.failed
      FROM components
               INNER JOIN summary_qualifications ON components.id = summary_qualifications.component_id AND components.visibility_change_set_pk = summary_qualifications.visibility_change_set_pk
      WHERE in_tenancy_and_visible_v1($1, $2, components)
      ORDER BY components.id, components.visibility_change_set_pk DESC, components.visibility_deleted_at DESC) AS a
