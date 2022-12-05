CREATE TYPE attribute_context_record_v1 AS
(
    attribute_context_prop_id              bigint,
    attribute_context_internal_provider_id bigint,
    attribute_context_external_provider_id bigint,
    attribute_context_component_id         bigint
);

CREATE OR REPLACE FUNCTION attribute_context_json_to_columns_v1(this_context jsonb,
                                                                OUT result attribute_context_record_v1)
AS
$$
BEGIN
    SELECT *
    FROM jsonb_to_record(this_context) AS x(
                                            attribute_context_prop_id bigint,
                                            attribute_context_internal_provider_id bigint,
                                            attribute_context_external_provider_id bigint,
                                            attribute_context_component_id bigint
        )
    INTO result;
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION in_attribute_context_v1(
    check_context jsonb,
    this_prop_id bigint,
    this_internal_provider_id bigint,
    this_external_provider_id bigint,
    this_component_id bigint
)
    RETURNS bool
    LANGUAGE sql
    IMMUTABLE
    PARALLEL SAFE
    CALLED ON NULL INPUT
AS
$$
SELECT
    -- All levels set
    (
        -- Least specific level check
            (
                -- PropId set
                    (
                            CASE
                                WHEN check_context -> 'attribute_context_prop_id' IS NULL OR
                                     check_context -> 'attribute_context_prop_id' = 'null'::jsonb THEN TRUE
                                ELSE (check_context -> 'attribute_context_prop_id')::bigint = this_prop_id
                                END
                            AND this_internal_provider_id = -1
                            AND this_external_provider_id = -1
                        )
                    -- InternalProviderId set
                    OR (
                                this_prop_id = -1
                            AND CASE
                                    WHEN check_context -> 'attribute_context_internal_provider_id' IS NULL OR
                                         check_context -> 'attribute_context_internal_provider_id' = 'null'::jsonb
                                        THEN TRUE
                                    ELSE (check_context -> 'attribute_context_internal_provider_id')::bigint =
                                         this_internal_provider_id
                                    END
                            AND this_external_provider_id = -1
                        )
                    -- ExternalProviderId set
                    OR (
                                this_prop_id = -1
                            AND this_internal_provider_id = -1
                            AND CASE
                                    WHEN check_context -> 'attribute_context_external_provider_id' IS NULL OR
                                         check_context -> 'attribute_context_external_provider_id' = 'null'::jsonb
                                        THEN TRUE
                                    ELSE (check_context -> 'attribute_context_external_provider_id')::bigint =
                                         this_external_provider_id
                                    END
                        )
                )
            -- Component check
            AND CASE
                    WHEN check_context -> 'attribute_context_component_id' IS NULL OR
                         check_context -> 'attribute_context_component_id' = 'null'::jsonb THEN TRUE
                    ELSE (check_context -> 'attribute_context_component_id')::bigint = this_component_id
                END
        )
        -- ComponentId not set
        OR (
        -- Least specific level check
            (
                -- PropId set
                    (
                            CASE
                                WHEN check_context -> 'attribute_context_prop_id' IS NULL OR
                                     check_context -> 'attribute_context_prop_id' = 'null'::jsonb THEN TRUE
                                ELSE (check_context -> 'attribute_context_prop_id')::bigint = this_prop_id
                                END
                            AND this_internal_provider_id = -1
                            AND this_external_provider_id = -1
                        )
                    -- InternalProviderId set
                    OR (
                                this_prop_id = -1
                            AND CASE
                                    WHEN check_context -> 'attribute_context_internal_provider_id' IS NULL OR
                                         check_context -> 'attribute_context_internal_provider_id' = 'null'::jsonb
                                        THEN TRUE
                                    ELSE (check_context -> 'attribute_context_internal_provider_id')::bigint =
                                         this_internal_provider_id
                                    END
                            AND this_external_provider_id = -1
                        )
                    -- ExternalProviderId set
                    OR (
                                this_prop_id = -1
                            AND this_internal_provider_id = -1
                            AND CASE
                                    WHEN check_context -> 'attribute_context_external_provider_id' IS NULL OR
                                         check_context -> 'attribute_context_external_provider_id' = 'null'::jsonb
                                        THEN TRUE
                                    ELSE (check_context -> 'attribute_context_external_provider_id')::bigint =
                                         this_external_provider_id
                                    END
                        )
                )
            -- Component check
            AND this_component_id = -1
        )
$$;

CREATE OR REPLACE FUNCTION exact_attribute_context_v1(check_context jsonb,
                                                      this_prop_id bigint,
                                                      this_internal_provider_id bigint,
                                                      this_external_provider_id bigint,
                                                      this_component_id bigint,
                                                      OUT result bool
)
AS
$$
DECLARE
    check_context_record       attribute_context_record_v1;
    prop_check                 bool;
    internal_provider_check    bool;
    external_provider_check    bool;
    least_specific_level_check bool;
    component_check            bool;
BEGIN
    RAISE DEBUG 'exact_attribute_context: % vs: p:% i:% e:% c:%',
        check_context,
        this_prop_id,
        this_internal_provider_id,
        this_external_provider_id,
        this_component_id;

    check_context_record := attribute_context_json_to_columns_v1(check_context);

    prop_check := CASE
                      WHEN check_context_record.attribute_context_prop_id IS NULL THEN
                          TRUE
                      ELSE
                          check_context_record.attribute_context_prop_id = this_prop_id
        END;
    RAISE DEBUG 'prop_check: %', prop_check;

    internal_provider_check := CASE
                                   WHEN check_context_record.attribute_context_internal_provider_id IS NULL THEN
                                       TRUE
                                   ELSE
                                           check_context_record.attribute_context_internal_provider_id =
                                           this_internal_provider_id
        END;
    RAISE DEBUG 'internal_provider_check: %', internal_provider_check;

    external_provider_check := CASE
                                   WHEN check_context_record.attribute_context_external_provider_id IS NULL THEN
                                       TRUE
                                   ELSE
                                           check_context_record.attribute_context_external_provider_id =
                                           this_external_provider_id
        END;
    RAISE DEBUG 'external_provider_check: %', external_provider_check;

    least_specific_level_check := prop_check OR internal_provider_check OR external_provider_check;

    component_check := (check_context_record.attribute_context_component_id = this_component_id);
    RAISE DEBUG 'component_check: %', component_check;

    result :=
            (least_specific_level_check AND component_check);
    RAISE DEBUG 'exact_attribute_context check result: %', result;
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION exact_attribute_context_v1(check_context jsonb,
                                                      reference record,
                                                      OUT result bool
)
AS
$$
BEGIN
    result := exact_attribute_context_v1(check_context,
                                         reference.attribute_context_prop_id,
                                         reference.attribute_context_internal_provider_id,
                                         reference.attribute_context_external_provider_id,
                                         reference.attribute_context_component_id);
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION exact_attribute_read_context_v1(check_context jsonb,
                                                           this_prop_id bigint,
                                                           this_internal_provider_id bigint,
                                                           this_external_provider_id bigint,
                                                           this_component_id bigint,
                                                           OUT result bool
)
AS
$$
DECLARE
    check_context_record       attribute_context_record_v1;
    prop_check                 bool;
    internal_provider_check    bool;
    external_provider_check    bool;
    least_specific_level_check bool;
    component_check            bool;
BEGIN
    RAISE DEBUG 'exact_attribute_read_context: % vs: p:% i:% e:% c:%',
        check_context,
        this_prop_id,
        this_internal_provider_id,
        this_external_provider_id,
        this_component_id;

    check_context_record := attribute_context_json_to_columns_v1(check_context);

    prop_check := CASE
                      WHEN check_context_record.attribute_context_prop_id IS NULL THEN
                          TRUE
                      ELSE
                          check_context_record.attribute_context_prop_id = this_prop_id
        END;
    RAISE DEBUG 'prop_check: %', prop_check;

    internal_provider_check := CASE
                                   WHEN check_context_record.attribute_context_internal_provider_id IS NULL THEN
                                       TRUE
                                   ELSE
                                           check_context_record.attribute_context_internal_provider_id =
                                           this_internal_provider_id
        END;
    RAISE DEBUG 'internal_provider_check: %', internal_provider_check;

    external_provider_check := CASE
                                   WHEN check_context_record.attribute_context_external_provider_id IS NULL THEN
                                       TRUE
                                   ELSE
                                           check_context_record.attribute_context_external_provider_id =
                                           this_external_provider_id
        END;
    RAISE DEBUG 'external_provider_check: %', external_provider_check;

    least_specific_level_check := (prop_check AND this_internal_provider_id = -1 AND this_external_provider_id = -1) OR
                                  (this_prop_id = -1 AND internal_provider_check AND this_external_provider_id = -1) OR
                                  (this_prop_id = -1 AND this_internal_provider_id = -1 AND external_provider_check);

    component_check := CASE
                           WHEN check_context_record.attribute_context_component_id IS NULL THEN
                               TRUE
                           ELSE
                               check_context_record.attribute_context_component_id = this_component_id
        END;
    RAISE DEBUG 'component_check: %', component_check;

    result :=
            (least_specific_level_check AND component_check);
    RAISE DEBUG 'in_attribute_context check result: %', result;
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION exact_attribute_read_context_v1(check_context jsonb,
                                                           reference record,
                                                           OUT result bool
)
AS
$$
BEGIN
    result := exact_attribute_read_context_v1(check_context,
                                              reference.attribute_context_prop_id,
                                              reference.attribute_context_internal_provider_id,
                                              reference.attribute_context_external_provider_id,
                                              reference.attribute_context_component_id);
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;


CREATE OR REPLACE FUNCTION exact_or_more_attribute_read_context_v1(
    check_context jsonb,
    this_prop_id bigint,
    this_internal_provider_id bigint,
    this_external_provider_id bigint,
    this_component_id bigint,
    OUT result bool
)
AS
$$
DECLARE
    check_context_record                  attribute_context_record_v1;
    prop_check                            bool;
    internal_provider_check               bool;
    external_provider_check               bool;
    least_specific_level_check            bool;
    least_specific_level_is_most_specific bool;
    component_check                       bool;
    component_is_most_specific            bool;
BEGIN
    RAISE DEBUG 'exact_or_more_attribute_read_context: % vs: p:% i:% e:% c:%',
        check_context,
        this_prop_id,
        this_internal_provider_id,
        this_external_provider_id,
        this_component_id;

    check_context_record := attribute_context_json_to_columns_v1(check_context);

    component_is_most_specific := FALSE;
    least_specific_level_is_most_specific := FALSE;

    IF check_context_record.attribute_context_component_id != -1 THEN
        component_is_most_specific := TRUE;
    ELSIF (check_context_record.attribute_context_prop_id != -1 OR
           check_context_record.attribute_context_internal_provider_id != -1 OR
           check_context_record.attribute_context_external_provider_id != -1) THEN
        least_specific_level_is_most_specific := TRUE;
    END IF;

    prop_check := CASE
                      WHEN check_context_record.attribute_context_prop_id IS NULL THEN
                          TRUE
                      ELSE
                          check_context_record.attribute_context_prop_id = this_prop_id
        END;
    RAISE DEBUG 'prop_check: %', prop_check;

    internal_provider_check := CASE
                                   WHEN check_context_record.attribute_context_internal_provider_id IS NULL THEN
                                       TRUE
                                   ELSE
                                           check_context_record.attribute_context_internal_provider_id =
                                           this_internal_provider_id
        END;
    RAISE DEBUG 'internal_provider_check: %', internal_provider_check;

    external_provider_check := CASE
                                   WHEN check_context_record.attribute_context_external_provider_id IS NULL THEN
                                       TRUE
                                   ELSE
                                           check_context_record.attribute_context_external_provider_id =
                                           this_external_provider_id
        END;
    RAISE DEBUG 'external_provider_check: %', external_provider_check;

    least_specific_level_check := (prop_check AND internal_provider_check AND external_provider_check);

    component_check := CASE
                           WHEN check_context_record.attribute_context_component_id IS NULL THEN
                               TRUE
                           ELSE
                               check_context_record.attribute_context_component_id = this_component_id
        END;
    RAISE DEBUG 'component_check: %', component_check;

    result := (least_specific_level_check AND component_check) OR
              CASE
                  WHEN
                      component_is_most_specific THEN (least_specific_level_check AND
                                                       component_check)
                  WHEN
                      least_specific_level_is_most_specific THEN (least_specific_level_check AND
                                                                  this_component_id != -1)
                  ELSE
                      FALSE
                  END;
    RAISE DEBUG 'exact_or_more_attribute_read_context check result: %', result;
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;

CREATE OR REPLACE FUNCTION exact_or_more_attribute_read_context_v1(check_context jsonb,
                                                                   reference record,
                                                                   OUT result bool
)
AS
$$
BEGIN
    result := exact_or_more_attribute_read_context_v1(check_context,
                                                      reference.attribute_context_prop_id,
                                                      reference.attribute_context_internal_provider_id,
                                                      reference.attribute_context_external_provider_id,
                                                      reference.attribute_context_component_id);
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;
