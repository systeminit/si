CREATE TABLE content_pairs
(
    created_at  timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    updated_at  timestamp with time zone  NOT NULL DEFAULT CLOCK_TIMESTAMP(),
    key         text                      NOT NULL,
    value       jsonb                     NOT NULL
);

CREATE UNIQUE INDEX unique_content_pairs ON content_pairs (key, value);

CREATE OR REPLACE FUNCTION content_pair_create_v1(
    this_key text,
    this_value jsonb
) RETURNS jsonb AS
$$
    INSERT INTO content_pairs (key, value)
    VALUES (this_key, this_value)
    RETURNING row_to_json(content_pairs) AS object;
$$ LANGUAGE SQL VOLATILE;
