CREATE TABLE change_batches 
(
    key               text                      NOT NULL PRIMARY KEY,
    sort_key          text                      NOT NULL,
    created_at        timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    value             bytea                     NOT NULL,
    serialization_lib text                      NOT NULL DEFAULT 'postcard'
);

CREATE INDEX IF NOT EXISTS change_batches_sort_key ON change_batches (sort_key);
