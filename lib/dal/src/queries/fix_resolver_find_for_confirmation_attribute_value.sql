SELECT row_to_json(fix_resolvers.*) as object
FROM fix_resolvers_v1($1, $2) AS fix_resolvers
WHERE fix_resolvers.attribute_value_id = $3
