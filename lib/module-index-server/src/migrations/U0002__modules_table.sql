CREATE TABLE modules
(
    id                          ident primary key default ident_create_v1(),
    name                        text                     NOT NULL,
    description                 text,
    owner_user_id               ident                    NOT NULL,
    owner_display_name          text,
    latest_hash                 char(64)                 NOT NULL,
    latest_hash_created_at      timestamp with time zone,
    created_at                  timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP()
);
