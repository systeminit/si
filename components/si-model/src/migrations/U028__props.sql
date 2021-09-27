CREATE TABLE props
(
    id         bigint PRIMARY KEY,
    si_id      text UNIQUE,
    schema_id  bigint                   NOT NULL REFERENCES schemas (id),
    name       text                     NOT NULL UNIQUE,
    parent_id  bigint REFERENCES props (id),
    is_item    bool                     NOT NULL DEFAULT false,
    kind       text                     NOT NULL,
    obj        jsonb                    NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

ALTER TABLE props
    ADD CONSTRAINT valid_kind_check CHECK (kind IN ('string', 'number', 'boolean', 'object', 'map', 'array'));

CREATE OR REPLACE FUNCTION prop_create_v1(
    name text,
    description text,
    this_kind text,
    this_parent_si_id text,
    this_schema_si_id text,
    this_is_item bool,
    OUT object jsonb
) AS
$$
DECLARE
    id             bigint;
    si_id          text;
    this_schema_id bigint;
    this_parent_id bigint;
    created_at     timestamp with time zone;
    updated_at     timestamp with time zone;
    si_storable    jsonb;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'prop:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT si_id_to_primary_key_v1(this_schema_si_id) INTO this_schema_id;

    SELECT jsonb_build_object(
                   'typeName', 'prop',
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
                   'kind', this_kind,
                   'isItem', this_is_item,
                   'schemaId', this_schema_si_id,
                   'siStorable', si_storable
               )
    INTO object;

    IF this_parent_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_parent_si_id) INTO this_parent_id;
        SELECT jsonb_set(object, '{parentId}', to_jsonb(this_parent_si_id), true) INTO object;
    END IF;

    INSERT INTO props (id, si_id, schema_id, name, parent_id, is_item, kind, obj)
    VALUES (id, si_id, this_schema_id, name, this_parent_id, this_is_item, this_kind, object);
END;
$$ LANGUAGE PLPGSQL;
