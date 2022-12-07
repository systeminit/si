SELECT object
FROM has_many_v1('key_pair_belongs_to_billing_account', $1, $2, 'key_pairs', $3)
ORDER BY (object ->> 'created_lamport_clock')::ident DESC
LIMIT 1;