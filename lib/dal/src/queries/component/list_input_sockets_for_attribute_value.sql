SELECT row_to_json(s.*) as object, e.id is not null as has_edge_connected
from attribute_value_belongs_to_attribute_prototype_v1($1, $2) avbtap
         inner join attribute_prototype_arguments_v1($1, $2) apa
                   on avbtap.belongs_to_id = apa.attribute_prototype_id
         inner join socket_belongs_to_internal_provider_v1($1, $2) sbtip
                   on sbtip.belongs_to_id = apa.internal_provider_id
         inner join sockets_v1($1, $2) s on sbtip.object_id = s.id
         left join edges_v1($1, $2) e on e.head_socket_id = s.id and e.head_object_id = $4
where avbtap.object_id = $3
