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
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION si_id_to_primary_key_or_null_v1(
    si_id text, OUT final_id bigint
) AS
$$
BEGIN
    IF si_id IS NOT NULL THEN
        SELECT id FROM si_id_to_primary_key_v1(si_id) INTO final_id;
    END IF;
END
$$ LANGUAGE PLPGSQL IMMUTABLE;
