CREATE TABLE update_clocks
(
    id           serial PRIMARY KEY,
    clock_si_id  text NOT NULL,
    epoch        bigint NOT NULL DEFAULT 0,
    update_count bigint NOT NULL DEFAULT 0
);

CREATE OR REPLACE FUNCTION update_clock_v1(this_clock_si_id text, OUT new_epoch bigint, OUT new_update_count bigint) AS
$$
DECLARE
    current_epoch        bigint;
    current_update_count bigint;
BEGIN
    SELECT epoch, update_count
    INTO current_epoch, current_update_count
    FROM update_clocks
    WHERE clock_si_id = this_clock_si_id FOR UPDATE;
    /* This is one less than the capacity of a raw javascript number - you're welcome */
    IF current_update_count = 9007199254740992 - 1 THEN
        IF current_epoch = 9007199254740992 - 1 THEN
            RAISE EXCEPTION 'epoch will overflow - so many updates - we are sorry. love, adam and fletcher - workspace id: %', this_workspace_id;
        END IF;
        UPDATE update_clocks AS u
        SET epoch        = u.epoch + 1,
            update_count = 0
        WHERE clock_si_id = this_clock_si_id
        RETURNING epoch, update_count INTO new_epoch, new_update_count;
    ELSE
        UPDATE update_clocks AS u
        SET update_count = u.update_count + 1
        WHERE clock_si_id = this_clock_si_id
        RETURNING epoch, update_count INTO new_epoch, new_update_count;
    END IF;
END;
$$ LANGUAGE PLPGSQL;
