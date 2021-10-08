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

SELECT resolver_create_v1('si:setEmptyObject', 'returns an empty object', 'emptyObject', 'object');
SELECT resolver_create_v1('si:setEmptyArray', 'returns an empty array', 'emptyArray', 'array');
SELECT resolver_create_v1('si:setEmptyMap', 'returns an empty map', 'emptyObject', 'object');
SELECT resolver_create_v1('si:setString', 'takes a string as input and returns it', 'string', 'string');
SELECT resolver_create_v1('si:setNumber', 'takes a number as input and returns it', 'number', 'number');
SELECT resolver_create_v1('si:setBoolean', 'takes a boolean as input and returns it', 'boolean', 'boolean');
SELECT resolver_create_v1('si:setObject', 'takes an object as input and returns it', 'object', 'object');
SELECT resolver_create_v1('si:setMap', 'takes a map as input and returns it', 'object', 'object');
SELECT resolver_create_v1('si:setArray', 'takes an array as input and returns it', 'array', 'array');
SELECT resolver_create_v1('si:unset', 'ensures this prop is never set', 'unset', 'unset');
SELECT resolver_create_v1('si:setJson', 'takes raw json as an input and returns it', 'json', 'json');
SELECT resolver_create_v1('si:setJs', 'takes a js function as an input and returns its output', 'js', 'json');

CREATE TABLE resolver_bindings
(
    id              bigint PRIMARY KEY,
    si_id           text UNIQUE,
    entity_id       bigint REFERENCES entities (id),
    system_id       bigint REFERENCES entities (id),
    resolver_id     bigint                   NOT NULL REFERENCES resolvers (id),
    schema_id       bigint                   NOT NULL REFERENCES schemas (id),
    prop_id         bigint REFERENCES props (id),
    parent_resolver_binding_id bigint REFERENCES resolver_bindings(id),
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
    this_parent_resolver_binding_si_id text,
    this_entity_si_id text,
    this_backend_binding jsonb,
    this_system_si_id text,
    this_change_set_si_id text,
    this_edit_session_si_id text,
    this_map_key_name text,
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
    this_parent_resolver_id bigint;
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
    SELECT si_id_to_primary_key_v1(this_schema_si_id) INTO this_schema_id;

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
                   'backendBinding', this_backend_binding,
                   'schemaId', this_schema_si_id,
                   'siStorable', si_storable
               )
    INTO object;

    IF this_prop_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_prop_si_id) INTO this_prop_id;
        SELECT jsonb_set(object, '{propId}', to_jsonb(this_prop_si_id), true) INTO object;
    END IF;

    IF this_parent_resolver_binding_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_parent_resolver_binding_si_id) INTO this_parent_resolver_id;
        SELECT jsonb_set(object, '{parentResolverBindingId}', to_jsonb(this_parent_resolver_binding_si_id), true) INTO object;
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

    IF this_map_key_name IS NOT NULL THEN
        SELECT jsonb_set(object, '{mapKeyName}', to_jsonb(this_map_key_name), true) INTO object;
    END IF;

    INSERT INTO resolver_bindings (id, si_id, entity_id, system_id, change_set_id, edit_session_id, resolver_id,
                                   schema_id, prop_id, parent_resolver_binding_id, obj, created_at, updated_at)
    VALUES (id, si_id, this_entity_id, this_system_id, this_change_set_id, this_edit_session_id, this_resolver_id,
            this_schema_id, this_prop_id, this_parent_resolver_id, object, created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL;

CREATE TABLE resolver_binding_values
(
    id                  bigint PRIMARY KEY,
    si_id               text UNIQUE,
    resolver_binding_id bigint                   NOT NULL REFERENCES resolver_bindings (id),
    resolver_id         bigint                   NOT NULL REFERENCES resolvers (id),
    schema_id           bigint                   NOT NULL REFERENCES schemas (id),
    entity_id           bigint REFERENCES entities (id),
    system_id           bigint REFERENCES entities (id),
    prop_id             bigint REFERENCES props (id),
    parent_resolver_binding_id bigint REFERENCES resolver_bindings(id),
    change_set_id       bigint REFERENCES change_sets (id),
    edit_session_id     bigint REFERENCES edit_sessions (id),
    output_value        jsonb                    NOT NULL,
    obj_value           jsonb                    NOT NULL,
    obj                 jsonb                    NOT NULL,
    created_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION resolver_binding_value_create_v1(
    this_output_value jsonb,
    this_obj_value jsonb,
    this_resolver_binding_si_id text,
    this_resolver_si_id text,
    this_schema_si_id text,
    this_prop_si_id text,
    this_parent_resolver_binding_si_id text,
    this_entity_si_id text,
    this_system_si_id text,
    this_change_set_si_id text,
    this_edit_session_si_id text,
    this_map_key_name text,
    OUT object jsonb
) AS
$$
DECLARE
    id                       bigint;
    si_id                    text;
    created_at               timestamp with time zone;
    updated_at               timestamp with time zone;
    this_resolver_binding_id bigint;
    this_resolver_id         bigint;
    this_schema_id           bigint;
    this_prop_id             bigint;
    this_parent_resolver_binding_id bigint;
    this_entity_id           bigint;
    this_system_id           bigint;
    this_change_set_id       bigint;
    this_edit_session_id     bigint;
    si_storable              jsonb;
BEGIN
    SELECT next_si_id_v1() INTO id;
    SELECT 'resolverBindingValue:' || id INTO si_id;
    SELECT NOW() INTO created_at;
    SELECT NOW() INTO updated_at;

    SELECT si_id_to_primary_key_v1(this_resolver_binding_si_id) INTO this_resolver_binding_id;
    SELECT si_id_to_primary_key_v1(this_resolver_si_id) INTO this_resolver_id;
    SELECT si_id_to_primary_key_v1(this_prop_si_id) INTO this_prop_id;
    SELECT si_id_to_primary_key_v1(this_schema_si_id) INTO this_schema_id;

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
                   'resolverBindingId', this_resolver_binding_si_id,
                   'resolverId', this_resolver_si_id,
                   'schemaId', this_schema_si_id,
                   'outputValue', this_output_value,
                   'objValue', this_obj_value,
                   'siStorable', si_storable
               )
    INTO object;

    IF this_prop_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_prop_si_id) INTO this_prop_id;
        SELECT jsonb_set(object, '{propId}', to_jsonb(this_prop_si_id), true) INTO object;
    END IF;

    IF this_parent_resolver_binding_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_parent_resolver_binding_si_id) INTO this_parent_resolver_binding_id;
        SELECT jsonb_set(object, '{parentResolverBindingId}', to_jsonb(this_parent_resolver_binding_si_id), true) INTO object;
    END IF;

    IF this_entity_si_id IS NOT NULL THEN
        SELECT si_id_to_primary_key_v1(this_entity_si_id) INTO this_entity_id;
        SELECT jsonb_set(object, '{entityId}', to_jsonb(this_entity_si_id), true) INTO object;
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

    IF this_map_key_name IS NOT NULL THEN
        SELECT jsonb_set(object, '{mapKeyName}', to_jsonb(this_map_key_name), true) INTO object;
    END IF;

    INSERT INTO resolver_binding_values (id, si_id, resolver_binding_id, resolver_id, schema_id, entity_id, system_id,
                                         prop_id, parent_resolver_binding_id, change_set_id, edit_session_id, output_value, obj_value, obj,
                                         created_at, updated_at)
    VALUES (id, si_id, this_resolver_binding_id, this_resolver_id, this_schema_id, this_entity_id, this_system_id,
            this_prop_id, this_parent_resolver_binding_id, this_change_set_id, this_edit_session_id, this_output_value, this_obj_value, object,
            created_at, updated_at);
END;
$$ LANGUAGE PLPGSQL;
