CREATE TABLE content_pairs
(
    key         text primary key          NOT NULL,
    created_at  timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    value       bytea                     NOT NULL
);
