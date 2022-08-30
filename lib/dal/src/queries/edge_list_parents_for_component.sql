SELECT DISTINCT ON (tail_object_id) tail_object_id,
                                    visibility_change_set_pk,
                                    visibility_deleted_at

FROM edges
WHERE in_tenancy_and_visible_v1($1, $2, edges)
  AND kind = 'configuration'
  AND head_object_kind = 'configuration'
  AND head_object_id = $3

ORDER BY tail_object_id DESC,
         id DESC,
         visibility_change_set_pk DESC,
         visibility_deleted_at DESC NULLS FIRST,
         head_node_id DESC,
         head_object_kind DESC,
         head_object_id DESC,
         head_socket_id DESC,
         tail_node_id DESC,
         tail_object_kind DESC,
         tail_socket_id DESC;
