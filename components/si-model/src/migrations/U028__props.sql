CREATE TABLE props
(
    id         bigint PRIMARY KEY,
    si_id      text UNIQUE,
    schema_id  bigint                   NOT NULL REFERENCES schemas (id),
    name       text                     NOT NULL UNIQUE,
    path       bigint[],
    kind       text                     NOT NULL,
    obj        jsonb                    NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

ALTER TABLE props
 ADD CONSTRAINT valid_kind_check CHECK (kind IN ('string'));

CREATE OR REPLACE FUNCTION prop_create_v1(
    name text,
    description text,
    this_kind text,
    path_si text[],
    schema_si_id text,
    OUT object jsonb
) AS
$$
DECLARE
    id             bigint;
    si_id          text;
    this_schema_id bigint;
    this_path      bigint[];
    created_at     timestamp with time zone;
    updated_at     timestamp with time zone;
    si_storable    jsonb;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'prop:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT si_id_to_primary_key_v1(schema_si_id) INTO this_schema_id;
    SELECT array_agg(si_id_to_primary_key_v1(path_entry)) FROM unnest(path_si) AS path_entry INTO this_path;

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
                   'schemaId', this_schema_id,
                   'path', path_si,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO props (id, si_id, schema_id, name, path, kind, obj)
        VALUES (id, si_id, this_schema_id, name, this_path, this_kind, object);
END;
$$ LANGUAGE PLPGSQL;
