SELECT row_to_json(components.*) AS object
  FROM components
  WHERE is_visible_v1(
      $1, 
      components.visibility_change_set_pk, 
      components.visibility_edit_session_pk, 
      components.visibility_deleted
    );
