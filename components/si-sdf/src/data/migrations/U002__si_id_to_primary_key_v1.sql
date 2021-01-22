CREATE OR REPLACE FUNCTION si_id_to_primary_key_v1(
    si_id text, OUT id bigint
) AS
$$
    BEGIN
        SELECT split_part(si_id, ':', 2)::bigint INTO id;
    EXCEPTION
        WHEN invalid_text_representation THEN
            RAISE EXCEPTION 'did you provide us with an id that has a namespace:numbers?: %', si_id;
    END
$$ LANGUAGE PLPGSQL IMMUTABLE
