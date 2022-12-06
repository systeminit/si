CREATE OR REPLACE FUNCTION attribute_value_create_appropriate_for_prototype_and_context_v1(
    this_write_tenancy jsonb,
    this_read_tenancy jsonb,
    this_visibility jsonb,
    this_attribute_prototype_id bigint,
    this_attribute_context jsonb,
    OUT new_attribute_value_ids bigint[]
) AS
$$
DECLARE
    attribute_value        attribute_values%ROWTYPE;
    tmp_attribute_value    attribute_values%ROWTYPE;
    new_attribute_value_id bigint;
BEGIN
    RAISE DEBUG 'attribute_value_create_appropriate_for_prototype_and_context_v1(%, %)',
        this_attribute_prototype_id,
        this_attribute_context;

    <<attribute_value_is_appropriate_check>>
    FOR attribute_value IN
        SELECT *
        FROM attribute_values_v1(this_read_tenancy, this_visibility) AS av
                 INNER JOIN (
            SELECT object_id AS attribute_value_id
            FROM attribute_value_belongs_to_attribute_prototype_v1(this_read_tenancy, this_visibility)
            WHERE belongs_to_id = this_attribute_prototype_id
        ) AS avbtap ON avbtap.attribute_value_id = av.id
        WHERE in_attribute_context_v1(this_attribute_context, av)
        ORDER BY id
        LOOP
            -- Check if the AttributeValue is of the _exact_ AttributeContext that we're looking for.
            IF exact_attribute_context_v1(this_attribute_context, attribute_value) THEN
                RAISE DEBUG 'attribute_value_create_appropriate_for_prototype_and_context_v1: Found appropriate AttributeValue(%)', attribute_value;

                -- Nothing to do, since we found an AttributeValue that we would have tried to create.
                new_attribute_value_ids := array_append(new_attribute_value_ids, attribute_value.id);
                CONTINUE attribute_value_is_appropriate_check;
            END IF;

            -- If the AttributeValue that we found has any proxies (even indirectly) that are of the
            -- appropriate AttributeContext then there's nothing to do here, as the AttributeValue
            -- we'd like to create already exists.
            FOR tmp_attribute_value IN
                SELECT *
                FROM attribute_values_v1(this_read_tenancy, this_visibility) AS av
                WHERE id IN (
                    WITH RECURSIVE recursive_attribute_values
                                       AS (
                            SELECT attribute_value.id AS attribute_value_id
                            UNION ALL
                            SELECT av.id AS attribute_value_id
                            FROM attribute_values_v1(this_read_tenancy, this_visibility) AS av
                                     JOIN recursive_attribute_values ON av.proxy_for_attribute_value_id =
                                                                        recursive_attribute_values.attribute_value_id
                            WHERE in_attribute_context_v1(this_attribute_context, av)
                        )
                    SELECT attribute_value_id
                    FROM recursive_attribute_values
                )
                ORDER BY id
                LOOP
                    IF exact_attribute_context_v1(this_attribute_context,
                                                  attribute_context_from_record_v1(tmp_attribute_value)) THEN
                        -- One of the proxies for the AttributeValue we're looking at is of the correct
                        -- AttributeContext, so we don't need to create anything as it already exists.
                        RAISE DEBUG 'attribute_value_create_appropriate_for_prototype_and_context_v1: Found appropriate proxy AttributeValue(%)', tmp_attribute_value;

                        new_attribute_value_ids := array_append(new_attribute_value_ids, tmp_attribute_value.id);
                        CONTINUE attribute_value_is_appropriate_check;
                    END IF;
                END LOOP;

            -- We need to create an AttributeValue that is of the specific AttributeContext that is a proxy
            -- back to this AttributeValue, since it now "exists" in the more specific context (even though,
            -- it might have a different value due to the function that generates its value having different
            --- inputs).
            new_attribute_value_id := attribute_value_vivify_value_and_parent_values_raw_v1(
                    this_write_tenancy,
                    this_read_tenancy,
                    this_visibility,
                    this_attribute_context,
                    attribute_value.id,
                    false
                );
            PERFORM set_belongs_to_v1(
                    'attribute_value_belongs_to_attribute_prototype',
                    this_read_tenancy,
                    this_write_tenancy,
                    this_visibility,
                    new_attribute_value_id,
                    this_attribute_prototype_id
                );
            RAISE DEBUG 'attribute_value_create_appropriate_for_prototype_and_context_v1: Using AttributeValue(%) for AttributeValue(%) in AttributeContext(%)',
                new_attribute_value_id,
                attribute_value,
                this_attribute_context;

            new_attribute_value_ids := array_append(new_attribute_value_ids, new_attribute_value_id);
        END LOOP attribute_value_is_appropriate_check;
END;
$$ LANGUAGE PLPGSQL;

-- DROP TYPE IF EXISTS attribute_value_create_new_affected_values_record_v1;
-- CREATE TYPE attribute_value_create_new_affected_values_record_v1 AS
-- (
--     attribute_context  jsonb,
--     attribute_value_id bigint
-- );

-- 1. Record the "base" AttributeContext as the one of the starting AttributeValue, minus
--    the External/Internal/Prop portion.
-- 2. Find AttributePrototypes that use an InternalProvider that is directly affected by
--    the current AttributeValue. For an AttributeValue that is somewhere under a Map/Array
--    this means that we need to walk the Props towards the root until we find a Prop that
--    has an InternalProvider, and find AttributePrototypes that use the InternalProvider
--    we just found.
--   2.1. For each AttributePrototype found, check if it is associated with an AttributeValue
--        that matches:
--            `(Current AttributePrototype AttributeContext) || ("base" AttributeContext)`
--     2.1.1. If we find an AttributeValue that is _less_ specific than what we were looking for,
--            we'll need to create a new AttributeValue for that AttributeContext, and associate
--            it with the current AttributePrototype.
--     2.1.2. If the originally found AttributeValue has a parent AttributeValue, then we need to
--            `AttributeValue::vivify_value_and_parent_values_without_child_proxies` with
--            `parent_attribute_context` as `("base" AttributeContext) || (found parent PropId)`,
--            and using the found parent's AttributeValueId.
--   2.2. Add the found/created AttributeValue to the list of ones to start from on the next time
--        through the loop.
-- 3. Find AttributePrototypes that use an ExternalProvider that is directly affected by
--    the currrent AttributeValue.
--   3.1. For each AttributePrototype found, check if it is associated with an AttributeValue
--        that matches:
--            `(Current AttributePrototype AttributeContext) || ("base" AttributeContext) || (Current Prototype Context ComponentId)`
--        We need the ComponentId off the AttributePrototype as we have crossed a Component
--        boundary by following an `ExternalProvider -> InternalProvider` link.
--     3.2.1. If we find an AttributeValue that is _less_ specific than what we were looking for,
--            we'll need to create a new AttributeValue for that AttributeContext, and associate
--            it with the current AttributeContext.
--   3.2. Add the found/created AttributeValue to the list of ones to start from on the next time
--        through the loop.
-- 4. If the list of AttributeValues to start from on the next loop iteration is not empty, go
--    back to Step 2. Otherwise, we're done.
--
-- An AttributePrototype "uses" value from InternalProvider if ALL:
--   * AttributePrototype -> AttributePrototypeArguments -> InternnalProvider
--   * AttributePrototype is the _most specific_ where above is true
--   * AttributePrototype.context <= InternalProvider AttributeValue.context
CREATE OR REPLACE FUNCTION attribute_value_create_new_affected_values_v1(
    this_write_tenancy jsonb,
    this_read_tenancy jsonb,
    this_visibility jsonb,
    this_attribute_value_id bigint
) RETURNS void AS
$$
DECLARE
    attribute_prototype           attribute_prototypes%ROWTYPE;
    attribute_prototype_context   jsonb;
    attribute_prototype_id        bigint;
    attribute_prototype_ids       bigint[];
    attribute_value               attribute_values%ROWTYPE;
    base_attribute_context        jsonb;
    current_attribute_context     jsonb;
    current_attribute_value_id    bigint;
    current_attribute_value_ids   bigint[];
    current_internal_provider_id  bigint;
    desired_attribute_context     jsonb;
    head_component_id             bigint;
    head_schema_id                bigint;
    head_schema_variant_id        bigint;
    insertion_attribute_context   jsonb;
    attribute_prototype_argument  attribute_prototype_arguments%ROWTYPE;
    internal_provider_id          bigint;
    internal_provider             internal_providers%ROWTYPE;
    new_attribute_value_id        bigint;
    next_attribute_value_ids      bigint[];
    proxy_attribute_value         attribute_values%ROWTYPE;
    proxy_check_attribute_context jsonb;
    seen_attribute_value_ids      bigint[];
    source_attribute_value        attribute_values%ROWTYPE;
    tmp_attribute_context         jsonb;
    tmp_attribute_value           attribute_values%ROWTYPE;
BEGIN
    RAISE DEBUG 'attribute_value_create_new_affected_values_v1(%, %, %, %)',
        this_write_tenancy,
        this_read_tenancy,
        this_visibility,
        this_attribute_value_id;

    current_attribute_value_ids := ARRAY [this_attribute_value_id];
    -- Grab the AttributeContext that we're starting with, since there should be an
    -- AttributeValue for every dependent AttributeValue that is _at least_ as specific as
    -- this starting AttributeContext.
    --
    -- We ignore the Prop/InternalProvider/ExternalProvider part, because they are always
    -- going to be different on the AttributeValues that we look at. The Component is also
    -- going to be different as soon as we go across an
    -- `ExternalProvider -> InternalProvider` link, but we need to dynamically figure out
    -- what the appropriate ComponentId is to use as we go along.
    base_attribute_context := jsonb_build_object(
                                      'attribute_context_component_id', attribute_context_component_id
                                  )
                              FROM (
                                       SELECT attribute_context_component_id
                                       FROM attribute_values_v1(this_read_tenancy, this_visibility)
                                       WHERE id = this_attribute_value_id
                                   ) AS av;
    RAISE DEBUG 'attribute_value_create_new_affected_values_v1: base_attribute_context: %', base_attribute_context;

    LOOP
        RAISE DEBUG 'attribute_value_create_new_affected_values_v1: current_attribute_value_ids: %', current_attribute_value_ids;
        RAISE DEBUG 'attribute_value_create_new_affected_values_v1: seen_attribute_value_ids: %', seen_attribute_value_ids;
        EXIT WHEN current_attribute_value_ids IS NULL;
        seen_attribute_value_ids := array_cat(seen_attribute_value_ids, current_attribute_value_ids);

        FOREACH current_attribute_value_id IN ARRAY current_attribute_value_ids
            LOOP
                SELECT *
                INTO STRICT source_attribute_value
                FROM attribute_values_v1(this_read_tenancy, this_visibility)
                WHERE id = current_attribute_value_id;
                RAISE DEBUG 'attribute_value_create_new_affected_values_v1: source_attribute_value: %', source_attribute_value;

                -- The base_attribute_context should take precedence for everything in the AttributeContext
                -- _except_ for the ComponentId, which we should get from the current AttributeValue since
                -- the current AttributeValue may have crossed an ExternalProvider -> InternalProvider
                -- boundary into a new Component.
                current_attribute_context := attribute_context_from_record_v1(source_attribute_value)
                                                 || base_attribute_context
                    || jsonb_build_object(
                                                     'attribute_context_component_id',
                                                     source_attribute_value.attribute_context_component_id
                                                 );
                RAISE DEBUG 'attribute_value_create_new_affected_values_v1: current_attribute_context: %', current_attribute_context;

                IF source_attribute_value.attribute_context_prop_id != -1 THEN
                    -- AttributeValues that are directly for a Prop can only be used by implicit InternalProviders.
                    RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Found AttributeValue for Prop';
                    -- Need to make sure the direct (and parent) InternalProviders have AttributeValues for the exact context.
                    FOR current_internal_provider_id,
                        attribute_prototype_id,
                        tmp_attribute_context
                        IN
                        WITH RECURSIVE parent_prop_tree AS (
                            SELECT source_attribute_value.attribute_context_prop_id AS prop_id
                            UNION ALL
                            SELECT p.parent_prop_id AS prop_id
                            FROM (
                                     SELECT object_id     AS child_prop_id,
                                            belongs_to_id AS parent_prop_id
                                     FROM prop_belongs_to_prop_v1(this_read_tenancy, this_visibility)
                                 ) AS p
                                     JOIN parent_prop_tree ON p.child_prop_id = parent_prop_tree.prop_id
                        )
                        SELECT ip.id AS current_internal_provider_id,
                               ip.attribute_prototype_id,
                               current_attribute_context ||
                               jsonb_build_object(
                                       'attribute_context_prop_id', -1,
                                       'attribute_context_internal_provider_id', ip.id,
                                       'attribute_context_external_provider_id', -1
                                   ) AS tmp_attribute_context
                        FROM internal_providers_v1(this_read_tenancy, this_visibility) AS ip
                                 INNER JOIN parent_prop_tree ON parent_prop_tree.prop_id = ip.prop_id
                        LOOP
                            RAISE DEBUG 'attribute_value_create_new_affected_values_v1: current_attribute_context(%)', current_attribute_context;
                            RAISE DEBUG 'attribute_value_create_new_affected_values_v1: tmp_attribute_context(%)', tmp_attribute_context;

                            -- TODO Looks like we've got a mismatch between some Prop setup, and some InternalProvider setup, where we're setting some things specific to Prop only, and the InternalProvider value only existing starting at the SchemaVariant level is causing an issue.
                            -- Grab the most specific AttributeValue that we already have for this InternalProvider
                            SELECT DISTINCT ON (attribute_context_internal_provider_id) av.*
                            INTO STRICT tmp_attribute_value
                            FROM attribute_values_v1(this_read_tenancy, this_visibility) AS av
                            WHERE in_attribute_context_v1(tmp_attribute_context, av)
                            ORDER BY attribute_context_internal_provider_id DESC,
                                     attribute_context_prop_id DESC,
                                     attribute_context_external_provider_id DESC,
                                     attribute_context_component_id DESC,
                                     -- bools sort false first ascending.
                                     av.tenancy_universal;
                            IF exact_attribute_context_v1(tmp_attribute_context,
                                                          attribute_context_from_record_v1(tmp_attribute_value)) THEN
                                -- There's already an appropriate AttributeValue for the InternalProvider. Nothing to do.
                                RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Found appropriate pre-existing AttributeValue(%) for InternalProvider(%)',
                                    tmp_attribute_value,
                                    current_internal_provider_id;
                                next_attribute_value_ids :=
                                        array_append(next_attribute_value_ids, tmp_attribute_value.id);
                                CONTINUE;
                            END IF;

                            -- We don't need to worry about giving the newly created AttributeValue an appropriate AttributePrototype as
                            -- implicit InternalProviders operate based off of their prop_id.
                            next_attribute_value_ids := array_append(
                                    next_attribute_value_ids,
                                    attribute_value_update_for_context_without_child_proxies_v1(
                                            this_write_tenancy,
                                            this_read_tenancy,
                                            this_visibility,
                                            tmp_attribute_value.id,
                                            NULL, -- No parent
                                            tmp_attribute_context,
                                            NULL, -- 'Unset' value
                                            NULL -- No key
                                        )
                                );
                        END LOOP;
                ELSIF source_attribute_value.attribute_context_internal_provider_id != -1 THEN
                    -- AttributeValues that are for an InternalProvider can be used by either of:
                    --   * An ExternalProvider
                    --   * An AttributeValue that is directly for a Prop.

                    RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Found AttributeValue for InternalProvider';
                    current_internal_provider_id := source_attribute_value.attribute_context_internal_provider_id;
                    tmp_attribute_context := base_attribute_context
                        || jsonb_build_object(
                                                     'attribute_context_component_id',
                                                     source_attribute_value.attribute_context_component_id
                                                 );
                    -- AttributePrototypes that refer to this InternalProvider (through AttributePrototypeArguments)
                    RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Looking for AttributePrototype that use InternalProvider(%) in AttributeContext(%), Tenancy(%), Visibility(%)',
                        current_internal_provider_id,
                        tmp_attribute_context,
                        this_read_tenancy,
                        this_visibility;
                    FOR attribute_prototype IN
                        SELECT ap.*
                        FROM attribute_prototypes_v1(this_read_tenancy, this_visibility) AS ap
                                 INNER JOIN attribute_prototype_arguments_v1(this_read_tenancy, this_visibility) AS apa
                                            ON ap.id = apa.attribute_prototype_id
                                                AND apa.internal_provider_id = current_internal_provider_id
                        LOOP
                            RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Found AttributePrototype(%)', attribute_prototype;
                            IF NOT in_attribute_context_v1(tmp_attribute_context, attribute_prototype) THEN
                                RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Does not have appropriate AttributeContext';
                                CONTINUE;
                            END IF;
                            RAISE DEBUG 'attribute_value_create_new_affected_values_v1: AttributePrototype(%) uses InternalProvider(%)', attribute_prototype, current_internal_provider_id;
                            desired_attribute_context := attribute_context_from_record_v1(attribute_prototype) ||
                                                         tmp_attribute_context;

                            IF attribute_prototype.attribute_context_prop_id != -1 THEN
                                -- This AttributePrototype is directly associated with a Prop
                                RAISE DEBUG 'attribute_value_create_new_affected_values_v1: AttributePrototype(%) is for Prop(%)',
                                    attribute_prototype.id,
                                    attribute_prototype.attribute_context_prop_id;

                                -- For each AttributeValue for this AttributePrototype, check if it is the level of specificity that we want,
                                -- if not, check the AttributeValue that is a proxy for the AttributeValue that we just looked at. Repeat
                                -- checking proxy AttributeValues until we either find an AttributeValue of the appropriate AttributeContext
                                -- (at which point, we're done, and can move on to the next "starting" AttributeValue), or we've run out of
                                -- proxy AttributeValues (at which point we need to create a new one with
                                -- attribute_value_update_for_context_raw_v1(...)).
                                <<attribute_values_for_prototype>>
                                FOR tmp_attribute_value IN
                                    SELECT av.*
                                    FROM attribute_values_v1(this_read_tenancy, this_visibility) AS av
                                             INNER JOIN attribute_value_belongs_to_attribute_prototype_v1(
                                            this_read_tenancy, this_visibility) AS avbtap
                                                        ON avbtap.object_id = av.id
                                                            AND avbtap.belongs_to_id = attribute_prototype.id
                                    WHERE in_attribute_context_v1(desired_attribute_context, av)
                                    ORDER BY av.id
                                    LOOP
                                        RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Checking if AttributeValue(%) is of desired AttributeContext(%)',
                                            tmp_attribute_value,
                                            desired_attribute_context;
                                        -- Check if this AttributeValue is of the AttributeContext that we want.
                                        IF exact_attribute_context_v1(desired_attribute_context,
                                                                      attribute_context_from_record_v1(tmp_attribute_value)) THEN
                                            RAISE DEBUG 'attribute_value_create_new_affected_values_v1: AttributeValue(%) is appropriate', tmp_attribute_value;
                                            next_attribute_value_ids :=
                                                    array_append(next_attribute_value_ids, tmp_attribute_value.id);
                                            CONTINUE;
                                        END IF;

                                        -- It wasn't, but there might be an AttributeValue that is a proxy for it that could be of the
                                        -- AttributeContext that we need one to exist for. We need to follow the proxy chain here,
                                        -- because the proxy might be overridden to have a different AttributePrototype. If that's the
                                        -- case, we still want to consider it as existing for the purposes of _this_
                                        -- AttributePrototype, and not create a new AttributeValue in the specific AttributeContext.
                                        FOR proxy_attribute_value IN
                                            WITH RECURSIVE proxy_attribute_values AS (
                                                SELECT *
                                                FROM attribute_values_v1(this_read_tenancy, this_visibility) AS av
                                                WHERE av.proxy_for_attribute_value_id = tmp_attribute_value.id
                                                UNION ALL
                                                SELECT av.*
                                                FROM attribute_values_v1(this_read_tenancy, this_visibility) AS av
                                                         JOIN proxy_attribute_values
                                                              ON av.proxy_for_attribute_value_id = proxy_attribute_values.id
                                            )
                                            SELECT *
                                            FROM proxy_attribute_values
                                            LOOP
                                                RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Checking if proxy AttributeValue(%) is of desired AttributeContext(%)',
                                                    proxy_attribute_value,
                                                    desired_attribute_context;
                                                IF exact_attribute_context_v1(desired_attribute_context,
                                                                              attribute_context_from_record_v1(proxy_attribute_value)) THEN
                                                    RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Proxy AttributeValue(%) is appropriate', proxy_attribute_value;
                                                    next_attribute_value_ids :=
                                                            array_append(next_attribute_value_ids, proxy_attribute_value.id);
                                                    CONTINUE attribute_values_for_prototype;
                                                END IF;
                                            END LOOP;

                                        RAISE DEBUG 'attribute_value_create_new_affected_values_v1: No appropriate AttributeValue found, creating new one.';
                                        -- The AttributeValue for an affected AttributePrototype, but there isn't a version of it that
                                        -- is of the AttributeContext that we're interested in, so we should make one.
                                        new_attribute_value_id :=
                                                attribute_value_vivify_value_and_parent_no_child_proxies_v1(
                                                        this_write_tenancy,
                                                        this_read_tenancy,
                                                        this_visibility,
                                                        desired_attribute_context,
                                                        tmp_attribute_value.id
                                                    );
                                        RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Setting prototype of AttributeValue(%) to AttributePrototype(%)',
                                            new_attribute_value_id,
                                            attribute_prototype.id;
                                        -- We need to make sure the new AttributeValue is associated with the correct AttributePrototype
                                        -- so that we can find it when we go through to update affected AttributeValues.
                                        PERFORM set_belongs_to_v1(
                                                'attribute_value_belongs_to_attribute_prototype',
                                                this_read_tenancy,
                                                this_write_tenancy,
                                                this_visibility,
                                                new_attribute_value_id,
                                                attribute_prototype.id
                                            );

                                        next_attribute_value_ids := array_append(
                                                next_attribute_value_ids,
                                                new_attribute_value_id
                                            );
                                    END LOOP attribute_values_for_prototype;
                            ELSIF attribute_prototype.attribute_context_external_provider_id != -1 THEN
                                insertion_attribute_context := attribute_context_from_record_v1(attribute_prototype)
                                                                   || base_attribute_context
                                    || jsonb_build_object(
                                                                       'attribute_context_component_id',
                                                                       source_attribute_value.attribute_context_component_id
                                                                   );
                                RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Ensuring AttributeValue exists for ExternalProvider at AttributeContext(%)', insertion_attribute_context;
                                -- This AttributePrototype is directly associated with an ExternalProvider
                                next_attribute_value_ids := array_cat(
                                        next_attribute_value_ids,
                                        attribute_value_create_appropriate_for_prototype_and_context_v1(
                                                this_write_tenancy,
                                                this_read_tenancy,
                                                this_visibility,
                                                attribute_prototype.id,
                                                insertion_attribute_context
                                            )
                                    );
                            ELSE
                                RAISE 'attribute_value_create_new_affected_values_v1: Don''t know how to handle an AttributePrototype(%) that isn''t for a Prop, or an ExternalProvider, and gets it''s data from an InternalProvider.', attribute_prototype;
                            END IF;

                        END LOOP;
                ELSIF source_attribute_value.attribute_context_external_provider_id != -1 THEN
                    RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Found AttributeValue for ExternalProvider(%)',
                        source_attribute_value.attribute_context_external_provider_id;
                    -- This AttributeValue is directly for an ExternalProvider.

                    -- Only InternalProviders can use ExternalProviders, so those are what we need to be looking for
                    -- on what will be using this AttributeValue.
                    tmp_attribute_context := attribute_context_from_record_v1(source_attribute_value)
                        || jsonb_build_object(
                                                     'attribute_context_prop_id', -1,
                                                     'attribute_context_internal_provider_id', NULL,
                                                     'attribute_context_external_provider_id', -1,
                                                 -- The InternalProviders that use this Attribute value can be for any Component,
                                                 -- and will not likely have the same ComponentId as the source AttributeValue
                                                     'attribute_context_component_id', NULL
                                                 );
                    RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Looking in AttributeContext(%)', tmp_attribute_context;
                    -- AttributePrototypes that "use" this ExternalProvider will have the ExternalProviderId set AND have the
                    -- same tail_component_id as source_attribute_value.attribute_context_component_id.
                    FOR head_schema_id,
                        head_schema_variant_id,
                        head_component_id,
                        internal_provider_id,
                        attribute_prototype_id
                        IN
                        SELECT cbts.belongs_to_id                     AS head_schema_id,
                               cbtsv.belongs_to_id                    AS head_schema_variant_id,
                               apa.head_component_id,
                               attribute_context_internal_provider_id AS internal_provider_id,
                               ap.id                                  AS attribute_prototype_id
                        FROM attribute_prototypes_v1(this_read_tenancy, this_visibility) AS ap
                                 INNER JOIN attribute_prototype_arguments_v1(this_read_tenancy, this_visibility) AS apa
                                            ON apa.attribute_prototype_id = ap.id
                                                AND external_provider_id =
                                                    source_attribute_value.attribute_context_external_provider_id
                                                AND
                                               tail_component_id = source_attribute_value.attribute_context_component_id
                                 INNER JOIN component_belongs_to_schema_v1(this_read_tenancy, this_visibility) AS cbts
                                            ON cbts.object_id = apa.head_component_id
                                 INNER JOIN component_belongs_to_schema_variant_v1(this_read_tenancy,
                                                                                   this_visibility) cbtsv
                                            ON cbtsv.object_id = apa.head_component_id
                        WHERE in_attribute_context_v1(tmp_attribute_context, ap)
                        LOOP
                            RAISE DEBUG 'attribute_value_create_new_affected_values_v1: AttributePrototype(%) uses ExternalProvider(%)', attribute_prototype, source_attribute_value.attribute_context_external_provider_id;
                            -- The AttributeContext that we want to ensure an AttributeValue exists for will be identical to the
                            -- source AttributeValue, except that it will be for an InternalProvider, instead of an ExternalProvider,
                            -- and that it will be associated with a different Component.
                            insertion_attribute_context := tmp_attribute_context
                                || jsonb_build_object(
                                                                   'attribute_context_external_provider_id', -1,
                                                                   'attribute_context_internal_provider_id',
                                                                   internal_provider_id,
                                                                   'attribute_context_component_id', head_component_id
                                                               );

                            RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Source AttributeValue(%) destination AttributeContext(%)',
                                source_attribute_value,
                                insertion_attribute_context;
                            next_attribute_value_ids := array_cat(
                                    next_attribute_value_ids,
                                    attribute_value_create_appropriate_for_prototype_and_context_v1(
                                            this_write_tenancy,
                                            this_read_tenancy,
                                            this_visibility,
                                            attribute_prototype_id,
                                            insertion_attribute_context
                                        )
                                );
                        END LOOP;
                ELSE
                    -- No idea what kind of AttributeValue this is, but it can't be good.
                    RAISE 'attribute_value_create_new_affected_values_v1: Found an AttributeValue(%) of unknown type. Tenancy(%), Visibility(%)',
                        source_attribute_value.id,
                        this_read_tenancy,
                        this_visibility;
                END IF;
            END LOOP;

        -- Set up current_attribute_value_ids to be the ones we found on this pass through the loop,
        -- that we haven't already seen, so we can start looking at them.
        current_attribute_value_ids := array_agg(x)
                                       FROM unnest(next_attribute_value_ids) AS x
                                       WHERE x != all (seen_attribute_value_ids);
        RAISE DEBUG 'attribute_value_create_new_affected_values_v1: Checking AttributeValueIds on next loop: %', current_attribute_value_ids;
        next_attribute_value_ids := NULL;
    END LOOP;
    RAISE DEBUG 'attribute_value_create_new_affected_values_v1: DONE';
END;
$$ LANGUAGE PLPGSQL;
