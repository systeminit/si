CREATE TABLE jwt_keys
(
    id          bigint PRIMARY KEY,
    public_key  text,
    private_key text,
    nonce       bytea,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION jwt_key_create_v1(public_key text,
                                             private_key text,
                                             nonce bytea,
                                             OUT id bigint) AS
$$
BEGIN
    SELECT next_si_id_v1() INTO id;
    INSERT INTO jwt_keys VALUES (id, public_key, private_key, nonce, DEFAULT, DEFAULT);
END;
$$ LANGUAGE PLPGSQL;


