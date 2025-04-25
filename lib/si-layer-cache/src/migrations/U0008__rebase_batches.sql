CREATE TABLE rebase_batches 
(
    key               text                      NOT NULL PRIMARY KEY,
    sort_key          text                      NOT NULL,
    created_at        timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    value             bytea                     NOT NULL,
    serialization_lib text                      NOT NULL DEFAULT 'postcard'
);

CREATE INDEX IF NOT EXISTS rebase_batches_sort_key ON rebase_batches (sort_key);
