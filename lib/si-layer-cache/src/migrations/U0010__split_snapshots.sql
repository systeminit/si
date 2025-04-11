CREATE TABLE split_snapshot_supergraphs
(
    key               text                      NOT NULL PRIMARY KEY,
    sort_key          text                      NOT NULL,
    created_at        timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    value             bytea                     NOT NULL,
    serialization_lib text                      NOT NULL DEFAULT 'postcard'
);

CREATE INDEX IF NOT EXISTS split_snapshot_supergraphs_sort_key ON split_snapshot_supergraphs (sort_key);


CREATE TABLE split_snapshot_subgraphs
(
    key               text                      NOT NULL PRIMARY KEY,
    sort_key          text                      NOT NULL,
    created_at        timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    value             bytea                     NOT NULL,
    serialization_lib text                      NOT NULL DEFAULT 'postcard'
);

CREATE INDEX IF NOT EXISTS split_snapshot_supergraphs_sort_key ON split_snapshot_subgraphs (sort_key);

CREATE TABLE split_snapshot_rebase_batches
(
    key               text                      NOT NULL PRIMARY KEY,
    sort_key          text                      NOT NULL,
    created_at        timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    value             bytea                     NOT NULL,
    serialization_lib text                      NOT NULL DEFAULT 'postcard'
);

CREATE INDEX IF NOT EXISTS split_snpapshot_rebase_batches_sort_key ON split_snapshot_rebase_batches (sort_key);
