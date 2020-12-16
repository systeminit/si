CREATE SEQUENCE si_id_seq;

CREATE OR REPLACE FUNCTION next_si_id_v1(OUT result bigint) AS $$
DECLARE
    our_epoch bigint := 1608072994248;
    seq_id bigint;
    now_millis bigint;
    shard_id int := 0;
    max_shard_id bigint := 2048;
BEGIN
    SELECT nextval('si_id_seq') % max_shard_id INTO seq_id;
    SELECT FLOOR(EXTRACT(EPOCH FROM clock_timestamp()) * 1000) INTO now_millis;
    result := (now_millis - our_epoch) << 23;
    result := result | (shard_id << 10);
    result := result | (seq_id);
END;
$$ LANGUAGE PLPGSQL;
