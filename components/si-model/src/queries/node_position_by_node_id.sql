SELECT obj AS object
FROM node_positions
WHERE node_id = si_id_to_primary_key_v1($1);
