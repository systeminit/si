CREATE TABLE resolvers
(
    id          bigint PRIMARY KEY,
    si_id       text UNIQUE,
    name        text                     NOT NULL UNIQUE,
    backend     text                     NOT NULL,
    output_kind text                     NOT NULL,
    obj         jsonb                    NOT NULL,
    created_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION resolver_create_v1(
    this_name text,
    this_description text,
    this_backend text,
    this_output_kind text,
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
    SELECT 'resolver:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT jsonb_build_object(
                   'typeName', 'resolver',
                   'objectId', si_id,
                   'deleted', false,
                   'createdAt', created_at,
                   'updatedAt', updated_at
               )
    INTO si_storable;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'name', this_name,
                   'description', this_description,
                   'backend', this_backend,
                   'outputKind', this_output_kind,
                   'siStorable', si_storable
               )
    INTO object;

    INSERT INTO resolvers (id, si_id, name, backend, output_kind, obj, created_at, updated_at)
    VALUES (id, si_id, this_name, this_backend, this_output_kind, object, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL;

CREATE TABLE resolver_bindings
(
    id              bigint PRIMARY KEY,
    si_id           text UNIQUE,
    entity_id       bigint REFERENCES entities (id),
    system_id       bigint REFERENCES entities (id),
    resolver_id     bigint                   NOT NULL REFERENCES resolvers (id),
    schema_id       bigint REFERENCES schemas (id),
    prop_id         bigint REFERENCES props (id),
    change_set_id   bigint REFERENCES change_sets (id),
    edit_session_id bigint REFERENCES edit_sessions (id),
    obj             jsonb                    NOT NULL,
    created_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION resolver_binding_create_v1(
    this_resolver_si_id text,
    this_schema_si_id text,
    this_prop_si_id text,
    this_entity_si_id text,
    this_backend_binding jsonb,
    this_system_si_id text,
    this_change_set_si_id text,
    this_edit_session_si_id text,
    OUT object jsonb
) AS
$$
DECLARE
    id                   bigint;
    si_id                text;
    created_at           timestamp with time zone;
    updated_at           timestamp with time zone;
    this_resolver_id     bigint;
    this_schema_id       bigint;
    this_prop_id         bigint;
    this_entity_id       bigint;
    this_system_id       bigint;
    this_change_set_id   bigint;
    this_edit_session_id bigint;
    si_storable          jsonb;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'resolverBinding:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT si_id_to_primary_key_v1(this_resolver_si_id) INTO this_resolver_id;
    SELECT si_id_to_primary_key_v1(this_prop_si_id) INTO this_prop_id;

    SELECT jsonb_build_object(
                   'typeName', 'resolverBinding',
                   'objectId', si_id,
                   'deleted', false,
                   'createdAt', created_at,
                   'updatedAt', updated_at
               )
    INTO si_storable;

    SELECT jsonb_build_object(
                   'id', si_id,
                   'resolverId', this_resolver_si_id,
                   'propId', this_prop_si_id,
                   'backendBinding', this_backend_binding,
                   'siStorable', si_storable
               )
    INTO object;

    IF this_schema_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_schema_si_id) INTO this_schema_id;
        SELECT jsonb_set(object, '{schemaId}', to_jsonb(this_schema_si_id), true) INTO object;
    END IF;

    IF this_prop_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_prop_si_id) INTO this_prop_id;
        SELECT jsonb_set(object, '{propId}', to_jsonb(this_prop_si_id), true) INTO object;
    END IF;


    IF this_entity_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_entity_si_id) INTO this_entity_id;
        RAISE WARNING 'Seting the jsonb!: %', this_entity_si_id;
        SELECT jsonb_set(object, '{entityId}', to_jsonb(this_entity_si_id), true) INTO object;
        RAISE WARNING 'I did it!';
    END IF;

    IF this_system_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_system_si_id) INTO this_system_id;
        SELECT jsonb_set(object, '{systemId}', to_jsonb(this_system_si_id), true) INTO object;
    END IF;

    IF this_change_set_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_change_set_si_id) INTO this_change_set_id;
        SELECT jsonb_set(object, '{changeSetId}', to_jsonb(this_change_set_id), true) INTO object;
    END IF;

    IF this_edit_session_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_edit_session_si_id) INTO this_edit_session_id;
        SELECT jsonb_set(object, '{editSessionId}', to_jsonb(this_edit_session_id), true) INTO object;
    END IF;

    INSERT INTO resolver_bindings (id, si_id, entity_id, system_id, change_set_id, edit_session_id, resolver_id,
                                   schema_id, prop_id, obj, created_at, updated_at)
    VALUES (id, si_id, this_entity_id, this_system_id, this_change_set_id, this_edit_session_id, this_resolver_id,
            this_schema_id, this_prop_id, object, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL;

