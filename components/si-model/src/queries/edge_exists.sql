SELECT obj AS object FROM edges WHERE
    edges.tail_vertex_node_si_id = $1
     AND edges.tail_vertex_object_type = $2
     AND edges.tail_vertex_socket = $3
     AND edges.head_vertex_node_si_id = $4
     AND edges.head_vertex_object_type = $5
     AND edges.head_vertex_socket = $6;

