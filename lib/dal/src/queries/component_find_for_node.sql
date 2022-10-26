SELECT row_to_json(c.*) AS object
FROM components_v1($1, $2) AS c
INNER JOIN node_belongs_to_component_v1($1, $2) AS nbtc
        ON c.id = nbtc.belongs_to_id
            AND nbtc.object_id = $3
