-- Returns list of component/system_id tuples representing every compoonent
-- membership in every system
SELECT DISTINCT ON (components.id)
    row_to_json(components.*) AS object,
    edges.tail_object_id AS system_id
  FROM components, edges
  WHERE is_visible_v1(
      $1,
      components.visibility_change_set_pk,
      components.visibility_edit_session_pk,
      components.visibility_deleted
    )
    AND edges.head_object_kind = 'component'
    AND edges.kind = 'includes'
    AND edges.tail_object_kind = 'system'
    AND components.id = edges.head_object_id;
