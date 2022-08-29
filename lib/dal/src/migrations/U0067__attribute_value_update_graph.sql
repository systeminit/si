CREATE OR REPLACE FUNCTION attribute_value_affected_graph_v1(this_tenancy jsonb,
                                                          this_visibility jsonb,
                                                          this_attribute_value_id bigint)
RETURNS TABLE(
    attribute_value_id           bigint,
    dependent_attribute_value_id bigint
)
AS
$$
DECLARE
    attribute_value             attribute_values%ROWTYPE;
    attribute_value_id          bigint;
    current_attribute_value_ids bigint[];
    current_prop_id             bigint;
    external_provider_id        bigint;
    internal_provider_id        bigint;
    next_attribute_value_ids    bigint[];
    original_attribute_context  jsonb;
    seen_attribute_value_ids    bigint[];
    tmp_attribute_context       jsonb;
    tmp_internal_provider       internal_providers%ROWTYPE;
    tmp_record_id               bigint;
    tmp_record_ids              bigint[];
    tmp_debug_bool              boolean;
BEGIN
    RAISE DEBUG 'Finding graph of AttributeValues affected by AttributeValue(%)', this_attribute_value_id;
    current_attribute_value_ids := ARRAY[this_attribute_value_id];

    LOOP
        RAISE DEBUG 'Current set of AttributeValueIds: %', current_attribute_value_ids;
        -- If we haven't found any more AttributeValues that depend on the ones
        -- we were looking at in the previous iteration of the loop, we can stop.
        EXIT WHEN current_attribute_value_ids IS NULL;
        -- Record this new set of AttributeValueIds as having been "seen" so they
        -- don't end up in any future batches of "current" AttributeValueIds.
        seen_attribute_value_ids := array_cat(seen_attribute_value_ids, current_attribute_value_ids);

        FOREACH attribute_value_id IN ARRAY current_attribute_value_ids LOOP
            RAISE DEBUG 'Looking at AttributeValue(%)', attribute_value_id;

            SELECT DISTINCT ON (id) *
            INTO STRICT attribute_value
            FROM attribute_values
            WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_values)
                  AND id = attribute_value_id
            ORDER BY id,
                     visibility_change_set_pk DESC,
                     visibility_deleted_at DESC NULLS FIRST;

            -- If the attribute_context_prop_id != -1 then that means that this AttributeValue
            -- represents a value that is "directly" part of a Component's schema, either
            -- because it is a value set for an attribute, or because it is for an implicit
            -- InternalProvider that is the "summary" of the schema from that Prop down to the
            -- leaf nodes.
            IF attribute_value.attribute_context_prop_id != -1 THEN
                RAISE DEBUG 'AttributeValue(%) is for Prop(%)', attribute_value.id, attribute_value.attribute_context_prop_id;

                current_prop_id := attribute_value.attribute_context_prop_id;

                -- If there are any "unsealed" proxies to the current AttributeValue, then we
                -- need to consider them as needing an update.
                SELECT array_agg(id)
                INTO tmp_record_ids
                FROM (
                    SELECT DISTINCT ON (id) id
                    FROM attribute_values
                    WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_values)
                          AND sealed_proxy = FALSE
                          AND proxy_for_attribute_value_id = attribute_value.id
                    ORDER BY id,
                             visibility_change_set_pk DESC,
                             visibility_deleted_at DESC NULLS FIRST
                ) AS proxy_ids;

                IF tmp_record_ids IS NOT NULL THEN
                    RAISE DEBUG 'Found unsealed proxies: %', tmp_record_ids;
                    RAISE DEBUG 'AttributeValue(%) depend on AttributeValue(%)', tmp_record_ids, attribute_value.id;

                    RETURN QUERY SELECT target_id AS attribute_value_id,
                                        attribute_value_id AS dependent_attribute_value_id
                                 FROM unnest(tmp_record_ids) AS target_id;
                    -- Add these new AttributeValues to the ones we'll use in the next loop iteration.
                    next_attribute_value_ids := array_cat(next_attribute_value_ids, tmp_record_ids);
                END IF;

                internal_provider_id := closest_internal_provider_to_prop_v1(this_tenancy, this_visibility, current_prop_id);

                -- Find the AttributeValue for the InternalProvider that we just found that is
                -- for the exact same AttributeContext as the AttributeValue that caused us to
                -- find it. This AttributeValue directly depends on the AttributeValue we are
                -- currently looking at.
                tmp_attribute_context := jsonb_build_object('attribute_context_prop_id',              -1,
                                                            'attribute_context_external_provider_id', -1,
                                                            'attribute_context_internal_provider_id', internal_provider_id,
                                                            'attribute_context_schema_id',            attribute_value.attribute_context_schema_id,
                                                            'attribute_context_schema_variant_id',    attribute_value.attribute_context_schema_variant_id,
                                                            'attribute_context_component_id',         attribute_value.attribute_context_component_id,
                                                            'attribute_context_system_id',            attribute_value.attribute_context_system_id);

                SELECT DISTINCT ON (id) id
                INTO STRICT tmp_record_id
                FROM attribute_values
                WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_values)
                      AND exact_attribute_context_v1(tmp_attribute_context, attribute_values)
                      AND attribute_context_internal_provider_id = internal_provider_id
                ORDER BY id,
                         visibility_change_set_pk DESC,
                         visibility_deleted_at DESC NULLS FIRST;

                RAISE DEBUG 'Found InternalProvider value(s): %', tmp_record_id;
                RAISE DEBUG 'AttributeValue(%) depend on AttributeValue(%)', ARRAY[tmp_record_id], attribute_value.id;

                RETURN QUERY SELECT tmp_record_id AS attribute_value_id,
                             attribute_value.id AS dependency_attribute_value_id;

                next_attribute_value_ids := array_append(next_attribute_value_ids, tmp_record_id);
            ELSIF attribute_value.attribute_context_internal_provider_id != -1 THEN
                -- We found an AttributeValue for an InternalProvider
                RAISE DEBUG 'AttributeValue(%) is InternalProvider(%)', attribute_value.id, attribute_value.attribute_context_internal_provider_id;

                -- Is there a parent Prop (and therefore an InternalProvider) that needs to be updated?
                SELECT DISTINCT ON (internal_providers.id) internal_providers.*
                INTO tmp_internal_provider
                FROM internal_providers
                INNER JOIN prop_belongs_to_prop ON prop_belongs_to_prop.belongs_to_id = internal_providers.prop_id
                                                   AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, prop_belongs_to_prop)
                INNER JOIN internal_providers AS child_internal_providers ON child_internal_providers.prop_id = prop_belongs_to_prop.object_id
                                                 AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, child_internal_providers)
                WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, internal_providers)
                      AND child_internal_providers.id = attribute_value.attribute_context_internal_provider_id
                ORDER BY internal_providers.id,
                         internal_providers.visibility_change_set_pk DESC,
                         internal_providers.visibility_deleted_at DESC NULLS FIRST;

                IF FOUND THEN
                    RAISE DEBUG 'Found a parent InternalProvider(%) for InternalProvider(%)', tmp_internal_provider.id, attribute_value.attribute_context_internal_provider_id;
                    tmp_attribute_context := jsonb_build_object('attribute_context_prop_id', -1,
                                                                'attribute_context_internal_provider_id', tmp_internal_provider.id,
                                                                'attribute_context_external_provider_id', -1,
                                                                'attribute_context_schema_id', attribute_value.attribute_context_schema_id,
                                                                'attribute_context_schema_variant_id', attribute_value.attribute_context_schema_variant_id,
                                                                'attribute_context_component_id', attribute_value.attribute_context_component_id,
                                                                'attribute_context_system_id', attribute_value.attribute_context_system_id);
                    -- TODO(jhelwig): This can, strictly speaking, find more AttributeValues that it considers
                    --                depending on this specific AttributeValue than there really are. The
                    --                problem is that we're not checking to see if there is a more appropriate
                    --                AttributeValue that this InternalProvider should be using, instead of
                    --                this (possibly less specific) AttributeValue.
                    SELECT array_agg(id)
                    INTO tmp_record_ids
                    FROM (
                        SELECT DISTINCT ON (id) id
                        FROM attribute_values
                        WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_values)
                              AND attribute_context_internal_provider_id = tmp_internal_provider.id
                              AND exact_or_more_attribute_read_context_v1(tmp_attribute_context, attribute_values)
                        ORDER BY id,
                                 attribute_values.visibility_change_set_pk DESC,
                                 attribute_values.visibility_deleted_at DESC NULLS FIRST
                    ) AS internal_provider_attribute_value_ids;

                    IF tmp_record_ids IS NOT NULL THEN
                        RAISE DEBUG 'Found AttributeValues for parent InternalProvider';
                        RAISE DEBUG 'AttributeValue(%) depend on AttributeValue(%)', tmp_record_ids, attribute_value.id;

                        RETURN QUERY SELECT target_id AS attribute_value_id,
                                            attribute_value.id AS dependency_attribute_value_id
                                     FROM unnest(tmp_record_ids) AS target_id;
                        next_attribute_value_ids := array_cat(next_attribute_value_ids, tmp_record_ids);
                    END IF;
                END IF;

                -- Which AttributePrototypes reference this InternalProvider, and therefore have AttributeValues
                -- that need to be updated.

                -- The AttributePrototypes could be associated with any of a Prop, an InternalProvider, or an,
                -- ExternalProvider, but they will always need to be within the same Component to be able to
                -- pull data from _this_ InternalProvider. (Which is why they're called _Internal_ Providers.)
                tmp_attribute_context := jsonb_build_object('attribute_context_prop_id', NULL,
                                                            'attribute_context_internal_provider_id', NULL,
                                                            'attribute_context_external_provider_id', NULL,
                                                            'attribute_context_schema_id', attribute_value.attribute_context_schema_id,
                                                            'attribute_context_schema_variant_id', attribute_value.attribute_context_schema_variant_id,
                                                            'attribute_context_component_id', attribute_value.attribute_context_component_id,
                                                            'attribute_context_system_id', attribute_value.attribute_context_system_id);

                SELECT array_agg(id)
                INTO tmp_record_ids
                FROM (
                    SELECT DISTINCT ON (attribute_values.id) attribute_values.id
                    FROM attribute_values
                    INNER JOIN attribute_value_belongs_to_attribute_prototype ON attribute_value_belongs_to_attribute_prototype.object_id = attribute_values.id
                                                                                 AND in_tenancy_and_visible_v1(this_tenancy,
                                                                                                               this_visibility,
                                                                                                               attribute_value_belongs_to_attribute_prototype)
                    INNER JOIN attribute_prototypes ON attribute_prototypes.id = attribute_value_belongs_to_attribute_prototype.belongs_to_id
                                                       AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_prototypes)
                    INNER JOIN attribute_prototype_arguments ON attribute_prototype_arguments.attribute_prototype_id = attribute_prototypes.id
                                                                AND in_tenancy_and_visible_v1(this_tenancy,
                                                                                              this_visibility,
                                                                                              attribute_prototype_arguments)
                    WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_values)
                          AND exact_or_more_attribute_read_context_v1(tmp_attribute_context, attribute_values)
                          AND attribute_prototype_arguments.internal_provider_id = attribute_value.attribute_context_internal_provider_id
                    ORDER BY attribute_values.id,
                             attribute_values.visibility_change_set_pk DESC,
                             attribute_values.visibility_deleted_at DESC NULLS FIRST
                ) AS new_attribute_value_ids;

                IF tmp_record_ids IS NOT NULL THEN
                    RAISE DEBUG 'Found prototype arguments that use this InternalProvider';
                    RAISE DEBUG 'AttributeValue(%) depend on AttributeValue(%)', tmp_record_ids, attribute_value.id;

                    RETURN QUERY SELECT target_id AS attribute_value_id,
                                        attribute_value.id AS dependency_attribute_value_id
                                 FROM unnest(tmp_record_ids) AS target_id;
                    next_attribute_value_ids := array_cat(next_attribute_value_ids, tmp_record_ids);
                END IF;
            ELSIF attribute_value.attribute_context_external_provider_id != -1 THEN
                -- We found an AttributeValue for an ExternalProvider
                RAISE DEBUG 'AttributeValue(%) is ExternalProvider(%)', attribute_value.id, attribute_value.attribute_context_external_provider_id;

                -- TODO(jhelwig): This combined with `exact_or_more_attribute_read_context_v1` isn't quite what we want.
                --                We need a function that can tell us:
                --                  * Is AttributeContext A set to the same level of specificity as AttributeContext B.
                --                  * Are any parts of the AttributeContext more specific than Component, that are set
                --                    in BOTH A & B identical.
                --                  * If any parts of the AttributeContext more specific than Component that are in B,
                --                    that are NOT in A, they count as being "or more".
                --
                -- Build up an AttributeReadContext appropriate for finding InternalProviders
                -- that are using this ExternalProvider as one of their arguments.
                tmp_attribute_context := jsonb_build_object('attribute_context_prop_id',              -1,
                                                            -- The InternalProvider is likely to be for some other Schema/Variant, and is
                                                            -- pretty much guaranteed to be for a different Component, so we need to be
                                                            -- looking at any possible values there. If this value for the ExternalProvider
                                                            -- is specific to a SystemId, then we need to make sure that we're only
                                                            -- considering AttributeValues that are also specific to that SystemId, but if
                                                            -- the AttributeValue for the ExternalProvider isn't specific to a SystemId,
                                                            -- then it doesn't matter whether or not the InternalProvider-side AttributeValue
                                                            -- is specific to a SystemId.
                                                            'attribute_context_internal_provider_id', NULL,
                                                            'attribute_context_external_provider_id', -1,
                                                            'attribute_context_schema_id',            NULL,
                                                            'attribute_context_schema_variant_id',    NULL,
                                                            'attribute_context_component_id',         NULL,
                                                            'attribute_context_system_id',            CASE WHEN attribute_value.attribute_context_system_id = -1 THEN
                                                                                                          NULL
                                                                                                      ELSE
                                                                                                          attribute_value.attribute_context_system_id
                                                                                                      END);

                -- TODO(jhelwig): This can, strictly speaking, find more AttributeValues that it considers
                --                depending on this specific AttributeValue for the ExternalProvider than
                --                there really are. The problem is that we're not checking to see if there
                --                is a more appropriate AttributeValue for this ExternalProvider that the
                --                InternalProvider on the other end of the connection should be using,
                --                instead of this (possibly less specific) AttributeValue.
                --
                -- Which InternalProviders reference this ExternalProvider in their arguments (and specifically
                -- the Component) that the AttributeValue is for.
                SELECT array_agg(id)
                INTO tmp_record_ids
                FROM (
                    SELECT DISTINCT ON (attribute_values.id) attribute_values.id
                    FROM attribute_values
                    INNER JOIN attribute_value_belongs_to_attribute_prototype
                        ON attribute_value_belongs_to_attribute_prototype.object_id = attribute_values.id
                           AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_value_belongs_to_attribute_prototype)
                    INNER JOIN attribute_prototypes
                        ON attribute_prototypes.id = attribute_value_belongs_to_attribute_prototype.belongs_to_id
                           AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_prototypes)
                    INNER JOIN attribute_prototype_arguments
                        ON attribute_prototype_arguments.attribute_prototype_id = attribute_prototypes.id
                           AND in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_prototype_arguments)
                           AND attribute_prototype_arguments.external_provider_id = attribute_value.attribute_context_external_provider_id
                           AND attribute_prototype_arguments.tail_component_id = attribute_value.attribute_context_component_id
                    WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, attribute_values)
                          -- See the TODO above tmp_attribute_context for why this is commented out.
                          --
                          -- AND exact_or_more_attribute_read_context_v1(tmp_attribute_context, attribute_values)
                    ORDER BY attribute_values.id,
                             attribute_values.visibility_change_set_pk DESC,
                             attribute_values.visibility_deleted_at DESC NULLS FIRST
                ) AS internal_provider_attribute_value_ids;

                IF tmp_record_ids IS NOT NULL THEN
                    RAISE DEBUG 'Found InternalProviders that use this ExternalProvider';
                    RAISE DEBUG 'AttributeValue(%) depend on AttributeValue(%)', tmp_record_ids, attribute_value.id;

                    RETURN QUERY SELECT target_id AS attribute_value_id,
                                        attribute_value.id AS dependency_attribute_value_id
                                 FROM unnest(tmp_record_ids) AS target_id;
                    next_attribute_value_ids := array_cat(next_attribute_value_ids, tmp_record_ids);
                END IF;
            ELSE
                -- No idea what we just found, but it can't be good.
                RAISE EXCEPTION 'Found an AttributeValue that we can''t determine the type of: %', attribute_value.id;
            END IF;
        END LOOP;

        -- Set up current_attribute_value_ids to be the ones we found on this pass through
        -- the loop, that we haven't already seen, so we can start looking at them.
        RAISE DEBUG 'Next AttributeValues to look at: %', next_attribute_value_ids;
        current_attribute_value_ids := array_agg(x) FROM unnest(next_attribute_value_ids) AS x
                                                    WHERE x != all(seen_attribute_value_ids);
        -- Clear out the "next" accumulator, so we don't add the new AttributeValues to the
        -- end of the list of ones we're looking through "currently" in the next iteration
        -- through the loop.
        next_attribute_value_ids := NULL;
    END LOOP;
END;
$$ LANGUAGE PLPGSQL;

CREATE OR REPLACE FUNCTION closest_internal_provider_to_prop_v1(this_tenancy jsonb,
                                                                this_visibility jsonb,
                                                                this_prop_id bigint,
                                                                OUT internal_provider_id bigint
)
AS
$$
DECLARE
  current_prop_id bigint;
BEGIN
    RAISE DEBUG 'Looking for InternalProvider for Prop(%)', this_prop_id;

    internal_provider_id := NULL;
    current_prop_id := this_prop_id;

    -- If the Prop for the initial AttributeValue doesn't have an associated
    -- InternalProvider, we need to keep looking towards the root of the Prop
    -- tree, until we find one.
    LOOP
        SELECT DISTINCT ON (id) id
        INTO STRICT internal_provider_id
        FROM internal_providers
        WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, internal_providers)
              AND internal_providers.prop_id = current_prop_id
        ORDER BY id,
                 visibility_change_set_pk DESC,
                 visibility_deleted_at DESC NULLS FIRST;

        IF FOUND THEN
            -- We found the closest InternalProvider between us & the root of the
            -- tree of Props, so there's no need to keep walking towards the root.
            EXIT;
        END IF;

        SELECT DISTINCT ON (id) belongs_to_id
        INTO STRICT current_prop_id
        FROM prop_belongs_to_prop
        WHERE in_tenancy_and_visible_v1(this_tenancy, this_visibility, prop_belongs_to_prop)
              AND object_id = current_prop_id
        ORDER BY id,
                 visibility_change_set_pk DESC,
                 visibility_deleted_at DESC NULLS FIRST;

        -- We somehow got to the root of the Prop tree without finding any
        -- InternalProviders. This is likely a bug in the configuration of
        -- the SchemaVariant.
        IF current_prop_id IS NULL THEN
            RAISE WARNING 'Unable to find an appropriate InternalProvider for Prop %', this_prop_id;
            EXIT;
        END IF;
    END LOOP;
END;
$$ LANGUAGE PLPGSQL IMMUTABLE;
