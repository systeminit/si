CREATE TYPE attribute_context_record_v1 AS
(
    attribute_context_prop_id           bigint,
    attribute_context_schema_id         bigint,
    attribute_context_schema_variant_id bigint,
    attribute_context_component_id      bigint,
    attribute_context_system_id         bigint
);

CREATE OR REPLACE FUNCTION attribute_context_json_to_columns_v1(this_context jsonb,
                                                                OUT result attribute_context_record_v1)
AS
$$
BEGIN
    SELECT *
    FROM jsonb_to_record(this_context) AS x(
                                            attribute_context_prop_id           bigint,
                                            attribute_context_schema_id         bigint,
                                            attribute_context_schema_variant_id bigint,
                                            attribute_context_component_id      bigint,
                                            attribute_context_system_id         bigint
                                            )
    INTO result;
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION in_attribute_context_v1(check_context jsonb,
                                                   this_prop_id bigint,
                                                   this_schema_id bigint,
                                                   this_schema_variant_id bigint,
                                                   this_component_id bigint,
                                                   this_system_id bigint,
                                                   OUT result bool
                                                   )
AS
$$
DECLARE
    check_context_record attribute_context_record_v1;
    prop_check bool;
    schema_check bool;
    schema_variant_check bool;
    component_check bool;
    system_check bool;
BEGIN
    RAISE DEBUG 'in_attribute_context: % vs: p:% s:% v:% c:% sys:%', check_context, this_prop_id, this_schema_id, this_schema_variant_id, this_component_id, this_system_id;

    check_context_record := attribute_context_json_to_columns_v1(check_context);

    prop_check := CASE
        WHEN check_context_record.attribute_context_prop_id IS NULL THEN
            TRUE
        ELSE
            check_context_record.attribute_context_prop_id = this_prop_id
    END;
    RAISE DEBUG 'prop__check: %', prop_check;

    schema_check := CASE
        WHEN check_context_record.attribute_context_schema_id IS NULL THEN
            TRUE
        ELSE
            check_context_record.attribute_context_schema_id = this_schema_id
    END;
    RAISE DEBUG 'schema_check: %', schema_check;

    schema_variant_check := CASE
        WHEN check_context_record.attribute_context_schema_variant_id IS NULL THEN
            TRUE
        ELSE
            check_context_record.attribute_context_schema_variant_id = this_schema_variant_id
    END;
    RAISE DEBUG 'schema_variant_check: %', schema_variant_check;

    component_check := CASE
        WHEN check_context_record.attribute_context_component_id IS NULL THEN
            TRUE
        ELSE
            check_context_record.attribute_context_component_id = this_component_id
    END;
    RAISE DEBUG 'component_check: %', component_check;

    system_check := CASE
        WHEN check_context_record.attribute_context_system_id IS NULL THEN
            TRUE
        ELSE
            check_context_record.attribute_context_system_id = this_system_id
    END;
    RAISE DEBUG 'system_check: %', system_check;

    -- This bottoms out just before saying "I want free-floating props", since those should be
    -- considered specially.  If we're interested in retrieving those, we're probably _only_
    -- interested in retrieving those, and we can accomplish that by explicitly setting the
    -- schema/variant/component/system to -1 in the incoming check_context.
    result := (prop_check AND schema_check AND schema_variant_check AND component_check AND system_check)
        OR (prop_check AND schema_check AND schema_variant_check AND component_check AND this_system_id = -1)
        OR (prop_check AND schema_check AND schema_variant_check AND this_component_id = -1 AND this_system_id = -1)
        OR (prop_check AND schema_check AND this_schema_variant_id = -1 AND this_component_id = -1 AND this_system_id = -1)
        OR CASE
            WHEN check_context_record.attribute_context_prop_id IS NULL THEN
                FALSE
            ELSE
                (prop_check AND this_schema_id = -1 AND this_schema_variant_id = -1 AND this_component_id = -1 AND this_system_id = -1)
        END;
    RAISE DEBUG 'in_attribute_context check result: %', result;
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION exact_attribute_context_v1(check_context jsonb,
                                                      this_prop_id bigint,
                                                      this_schema_id bigint,
                                                      this_schema_variant_id bigint,
                                                      this_component_id bigint,
                                                      this_system_id bigint,
                                                      OUT result bool
                                                      )
AS
$$
DECLARE
    check_context_record attribute_context_record_v1;
    prop_check bool;
    schema_check bool;
    schema_variant_check bool;
    component_check bool;
    system_check bool;
BEGIN
    RAISE DEBUG 'exact_attribute_context: % vs: p:% s:% v:% c:% sys:%', check_context, this_prop_id, this_schema_id, this_schema_variant_id, this_component_id, this_system_id;

    check_context_record := attribute_context_json_to_columns_v1(check_context);

    prop_check := (check_context_record.attribute_context_prop_id = this_prop_id);
    RAISE DEBUG 'prop_check: %', prop_check;

    schema_check := (check_context_record.attribute_context_schema_id = this_schema_id);
    RAISE DEBUG 'schema_check: %', schema_check;

    schema_variant_check := (check_context_record.attribute_context_schema_variant_id = this_schema_variant_id);
    RAISE DEBUG 'schema_variant_check: %', schema_variant_check;

    component_check := (check_context_record.attribute_context_component_id = this_component_id);
    RAISE DEBUG 'component_check: %', component_check;

    system_check := (check_context_record.attribute_context_system_id = this_system_id);
    RAISE DEBUG 'system_check: %', system_check;

    result := (prop_check AND schema_check AND schema_variant_check AND component_check AND system_check);
    RAISE DEBUG 'exact_attribute_context check result: %', result;
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION exact_attribute_read_context_v1(check_context jsonb,
                                                           this_prop_id bigint,
                                                           this_schema_id bigint,
                                                           this_schema_variant_id bigint,
                                                           this_component_id bigint,
                                                           this_system_id bigint,
                                                           OUT result bool
                                                           )
AS
$$
DECLARE
    check_context_record attribute_context_record_v1;
    prop_check bool;
    schema_check bool;
    schema_variant_check bool;
    component_check bool;
    system_check bool;
BEGIN
    RAISE DEBUG 'exact_attribute_read_context: % vs: p:% s:% v:% c:% sys:%', check_context, this_prop_id, this_schema_id, this_schema_variant_id, this_component_id, this_system_id;

    check_context_record := attribute_context_json_to_columns_v1(check_context);

    prop_check := CASE
        WHEN check_context_record.attribute_context_prop_id IS NULL THEN
            TRUE
        ELSE
            check_context_record.attribute_context_prop_id = this_prop_id
    END;
    RAISE DEBUG 'prop__check: %', prop_check;

    schema_check := CASE
        WHEN check_context_record.attribute_context_schema_id IS NULL THEN
            TRUE
        ELSE
            check_context_record.attribute_context_schema_id = this_schema_id
    END;
    RAISE DEBUG 'schema_check: %', schema_check;

    schema_variant_check := CASE
        WHEN check_context_record.attribute_context_schema_id IS NULL THEN
            TRUE
        ELSE
            check_context_record.attribute_context_schema_variant_id = this_schema_variant_id
    END;
    RAISE DEBUG 'schema_variant_check: %', schema_variant_check;

    component_check := CASE
        WHEN check_context_record.attribute_context_component_id IS NULL THEN
            TRUE
        ELSE
            check_context_record.attribute_context_component_id = this_component_id
    END;
    RAISE DEBUG 'component_check: %', component_check;

    system_check := CASE
        WHEN check_context_record.attribute_context_system_id IS NULL THEN
            TRUE
        ELSE
            check_context_record.attribute_context_system_id = this_system_id
    END;
    RAISE DEBUG 'system_check: %', system_check;

    result := (prop_check AND schema_check AND schema_variant_check AND component_check AND system_check);
    RAISE DEBUG 'in_attribute_context check result: %', result;
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;
