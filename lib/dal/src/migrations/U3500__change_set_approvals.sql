CREATE TABLE change_set_approvals
(
    id ident primary key NOT NULL DEFAULT ident_create_v1(),
    created_at timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    change_set_id ident NOT NULL,
    status text NOT NULL,
    user_id ident NOT NULL,
    checksum text NOT NULL
);
