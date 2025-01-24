CREATE TABLE change_set_approvals
(
    id ident primary key NOT NULL DEFAULT ident_create_v1(),
    created_at timestamp with time zone NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    change_set_id ident NOT NULL,
    status text NOT NULL,
    user_id ident NOT NULL,
    checksum text NOT NULL
);

CREATE VIEW latest_change_set_approvals AS
    SELECT DISTINCT ON (
        change_set_id,
        user_id,
        checksum
    ) * FROM change_set_approvals
    ORDER BY
        change_set_id,
        user_id,
        checksum,
        created_at
    DESC;
