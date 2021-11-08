CREATE TABLE jwt_keys
(
    pk          bigserial PRIMARY KEY,
    public_key  text,
    private_key text,
    nonce       bytea,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION jwt_key_create_v1(this_public_key text,
                                             this_private_key text,
                                             this_nonce bytea) RETURNS VOID AS
$$
BEGIN
    INSERT INTO jwt_keys (public_key, private_key, nonce) VALUES (this_public_key, this_private_key, this_nonce);
END;
$$ LANGUAGE PLPGSQL VOLATILE;


