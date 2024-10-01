CREATE TABLE cached_modules 
(
    id                          ident primary key default ident_create_v1(),
    schema_id                   ident                    NOT NULL,
    schema_name                 text                     NOT NULL,
    display_name                text,
    category                    text,
    link                        text,
    color                       text,
    description                 text,
    component_type              text                     NOT NULL,
    latest_hash                 text                     NOT NULL,
    created_at                  timestamp with time zone NOT NULL,
    package_data                bytea
);

CREATE INDEX IF NOT EXISTS latest_hash_idx ON cached_modules (latest_hash);
CREATE INDEX IF NOT EXISTS schema_id_idx ON cached_modules (schema_id);