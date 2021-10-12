CREATE TABLE schemas
(
    id          bigint PRIMARY KEY,
    si_id       text UNIQUE,
    name        text                     NOT NULL UNIQUE,
    entity_type text                     NOT NULL UNIQUE,
    description text                     NOT NULL,
    obj         jsonb                    NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION schema_create_v1(
    name text,
    entity_type text,
    description text,
    OUT object jsonb
) AS
$$
DECLARE
    id          bigint;
    si_id       text;
    created_at  timestamp with time zone;
    updated_at  timestamp with time zone;
    si_storable jsonb;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'schema:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;
    SELECT jsonb_build_object(
                   'typeName', 'schema',
                   'objectId', si_id,
                   'deleted', false,
                   'createdAt', created_at,
                   'updatedAt', updated_at
               )
    INTO si_storable;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', name,
                   'description', description,
                   'entityType', entity_type,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO schemas (id, si_id, name, entity_type, description, obj)
    VALUES (id, si_id, name, entity_type, description, object);
END;
$$ LANGUAGE PLPGSQL;

