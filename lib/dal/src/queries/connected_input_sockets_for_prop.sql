SELECT row_to_json(s.*) as object
from attribute_value_belongs_to_attribute_prototype_v1($1, $2) avbtap
         left join attribute_prototypes_v1($1, $2) ap
                   on ap.id = avbtap.belongs_to_id
         left join attribute_prototype_arguments_v1($1, $2) apa
                   on ap.id = apa.attribute_prototype_id
         left join socket_belongs_to_internal_provider_v1($1, $2) sbtip
                   on sbtip.belongs_to_id = apa.internal_provider_id
         left join sockets_v1($1, $2) s on sbtip.object_id = s.id
         left join edges_v1($1, $2) e on e.head_socket_id = s.id
where avbtap.object_id = $3
  and e.head_object_id = $4