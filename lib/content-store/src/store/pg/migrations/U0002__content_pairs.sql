CREATE TABLE content_pairs
(
    key         text primary key          NOT NULL,
    created_at  timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    value       jsonb                     NOT NULL
);

CREATE UNIQUE INDEX unique_content_pairs ON content_pairs (key, value);