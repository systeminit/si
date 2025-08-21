use dal::{
    ChangeSet,
    Component,
    DalContext,
    SchemaVariantId,
};
use dal_test::{
    Result,
    helpers::{
        ChangeSetTestHelpers,
        attribute::value,
        change_set,
        component,
        schema::variant,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use serde_json::json;

#[test]
async fn autosubscribe_basic_suggest_sources(ctx: &mut DalContext) -> Result<()> {
    // Test basic functionality where one component has a prop with "suggestSources"
    // pointing to another component's prop, and values match

    // Create source schema matching the working default_subscription pattern
    let _source_schema_id = variant::create(
        ctx,
        "TestSourceSchema",
        r#"
        function main() {
            return {
                props: [
                    {
                        name: "source_string",
                        kind: "string"
                    }
                ]
            };
        }
        "#,
    )
    .await?;

    // Create destination schema with suggestSources
    let _dest_schema_id = variant::create(
        ctx,
        "TestDestSchema",
        r#"
        function main() {
            return {
                props: [
                    {
                        name: "dest_string",
                        kind: "string",
                        suggestSources: [
                            { schema: "TestSourceSchema", prop: "/domain/source_string" }
                        ]
                    }
                ]
            };
        }
        "#,
    )
    .await?;

    // Create components from both schemas
    let source_component = component::create(ctx, "TestSourceSchema", "source").await?;
    let dest_component = component::create(ctx, "TestDestSchema", "dest").await?;
    change_set::commit(ctx).await?;

    // Set matching values on both components
    let test_value = "matching_value";
    value::set(ctx, (source_component, "/domain/source_string"), test_value).await?;
    value::set(ctx, (dest_component, "/domain/dest_string"), test_value).await?;
    change_set::commit(ctx).await?;

    // Call autosubscribe
    let result = Component::autosubscribe(ctx, dest_component).await?;

    change_set::commit(ctx).await?;

    // Verify subscription was created correctly
    assert_eq!(
        1,
        result.success_count(),
        "Should create exactly one subscription"
    );

    assert!(!result.has_issues(), "Should not have any issues");

    // Verify the details of the successful subscription
    assert_eq!(1, result.successful.len());
    let successful_sub = &result.successful[0];
    assert_eq!(test_value, successful_sub.matched_value.as_str().unwrap());
    // Verify the subscription actually works by changing source value
    let new_value = "updated_value";
    value::set(ctx, (source_component, "/domain/source_string"), new_value).await?;
    change_set::commit(ctx).await?;

    // Check that destination value updated
    let dest_value = value::get(ctx, (dest_component, "/domain/dest_string")).await?;
    assert_eq!(
        json!(new_value),
        dest_value,
        "Destination should reflect source change"
    );

    Ok(())
}

#[test]
async fn autosubscribe_basic_suggest_as_source_for(ctx: &mut DalContext) -> Result<()> {
    // Test basic functionality where one component has a prop with "suggestAsSourceFor"
    // pointing to another component's prop, and values match

    // Create destination schema with a simple string prop
    create_simple_schema(ctx, "DestSchema", "DestProp").await?;

    // Create source schema with a prop that suggests itself as source for dest
    create_schema_with_suggest_as_source_for(
        ctx,
        "SourceSchema",
        "SourceProp",
        "DestSchema",
        "/domain/DestProp",
    )
    .await?;

    // Create components from both schemas
    let source_component = component::create(ctx, "SourceSchema", "source").await?;
    let dest_component = component::create(ctx, "DestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set matching values on both components
    let test_value = "matching_value";
    value::set(ctx, (source_component, "/domain/SourceProp"), test_value).await?;
    value::set(ctx, (dest_component, "/domain/DestProp"), test_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe on the destination component
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Verify subscription was created
    assert_eq!(
        1,
        result.success_count(),
        "Should create exactly one subscription"
    );
    assert!(!result.has_issues(), "Should not have any issues");

    // Verify the details of the successful subscription
    assert_eq!(1, result.successful.len());
    let successful_sub = &result.successful[0];
    assert_eq!(test_value, successful_sub.matched_value.as_str().unwrap());

    // Verify the subscription actually works by changing source value
    let new_value = "updated_value";
    value::set(ctx, (source_component, "/domain/SourceProp"), new_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that destination value updated
    let dest_value = value::get(ctx, (dest_component, "/domain/DestProp")).await?;
    assert_eq!(
        json!(new_value),
        dest_value,
        "Destination should reflect source change"
    );

    Ok(())
}

#[test]
async fn autosubscribe_no_matching_values(ctx: &mut DalContext) -> Result<()> {
    // Test that subscriptions are NOT created when prop suggestions exist
    // but values don't match

    // Create source schema with a simple string prop
    create_simple_schema(ctx, "SourceSchema", "SourceProp").await?;

    // Create destination schema with a prop that suggests the source
    create_schema_with_suggest_sources(
        ctx,
        "DestSchema",
        "DestProp",
        "SourceSchema",
        "/domain/SourceProp",
    )
    .await?;

    // Create components from both schemas
    let source_component = component::create(ctx, "SourceSchema", "source").await?;
    let dest_component = component::create(ctx, "DestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set NON-matching values on both components
    value::set(
        ctx,
        (source_component, "/domain/SourceProp"),
        "source_value",
    )
    .await?;
    value::set(
        ctx,
        (dest_component, "/domain/DestProp"),
        "different_dest_value",
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Verify no subscriptions were created since values don't match
    assert_eq!(
        0,
        result.success_count(),
        "Should not create subscriptions when values don't match"
    );

    assert!(!result.has_issues(), "Should not have any issues");

    // Verify destination value remains unchanged
    let dest_value = value::get(ctx, (dest_component, "/domain/DestProp")).await?;
    assert_eq!(
        json!("different_dest_value"),
        dest_value,
        "Destination value should remain unchanged"
    );

    Ok(())
}

#[test]
async fn autosubscribe_multiple_matches_conflict(ctx: &mut DalContext) -> Result<()> {
    // Test conflict resolution when multiple components could be sources
    // for the same destination prop (should return conflicts, not create subscriptions)

    // Create source schema with a simple string prop
    create_simple_schema(ctx, "SourceSchema", "SourceProp").await?;

    // Create destination schema with a prop that suggests the source
    create_schema_with_suggest_sources(
        ctx,
        "DestSchema",
        "DestProp",
        "SourceSchema",
        "/domain/SourceProp",
    )
    .await?;

    // Create multiple source components with the same schema
    let source_component_1 = component::create(ctx, "SourceSchema", "source1").await?;
    let source_component_2 = component::create(ctx, "SourceSchema", "source2").await?;
    let dest_component = component::create(ctx, "DestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set matching values on all components - this creates ambiguity
    let test_value = "ambiguous_value";
    value::set(ctx, (source_component_1, "/domain/SourceProp"), test_value).await?;
    value::set(ctx, (source_component_2, "/domain/SourceProp"), test_value).await?;
    value::set(ctx, (dest_component, "/domain/DestProp"), test_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Verify no subscriptions were created due to conflict
    assert_eq!(
        0,
        result.success_count(),
        "Should not create subscriptions when multiple matches exist"
    );
    assert_eq!(0, result.error_count(), "Should not have any errors");

    // Verify conflicts are returned
    assert_eq!(
        1,
        result.conflict_count(),
        "Should have exactly one conflict"
    );
    assert!(result.has_issues(), "Should have issues due to conflicts");

    // Check the conflict details
    assert_eq!(1, result.conflicts.len());
    let conflict = &result.conflicts[0];
    assert_eq!(
        2,
        conflict.matches.len(),
        "Should have exactly 2 conflicting matches"
    );

    // Verify both conflict matches have the expected value
    for conflict_match in &conflict.matches {
        assert_eq!(test_value, conflict_match.value.as_str().unwrap());
    }

    // Verify destination value remains unchanged
    let dest_value = value::get(ctx, (dest_component, "/domain/DestProp")).await?;
    assert_eq!(
        json!(test_value),
        dest_value,
        "Destination value should remain as originally set"
    );

    Ok(())
}

#[test]
async fn autosubscribe_single_match_from_multiple_candidates(ctx: &mut DalContext) -> Result<()> {
    // Test that when multiple candidates exist but only one has matching values,
    // a subscription is created with that one

    // Create source schema with a simple string prop
    create_simple_schema(ctx, "SourceSchema", "SourceProp").await?;

    // Create destination schema with a prop that suggests the source
    create_schema_with_suggest_sources(
        ctx,
        "DestSchema",
        "DestProp",
        "SourceSchema",
        "/domain/SourceProp",
    )
    .await?;

    // Create multiple source components with the same schema
    let source_component_1 = component::create(ctx, "SourceSchema", "source1").await?;
    let source_component_2 = component::create(ctx, "SourceSchema", "source2").await?;
    let dest_component = component::create(ctx, "DestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set different values - only one will match the destination
    let dest_value = "matching_value";
    value::set(
        ctx,
        (source_component_1, "/domain/SourceProp"),
        "non_matching_value",
    )
    .await?;
    value::set(ctx, (source_component_2, "/domain/SourceProp"), dest_value).await?;
    value::set(ctx, (dest_component, "/domain/DestProp"), dest_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Verify exactly one subscription was created
    assert_eq!(
        1,
        result.success_count(),
        "Should create exactly one subscription"
    );

    assert!(!result.has_issues(), "Should not have any issues");

    // Verify the subscription details
    assert_eq!(1, result.successful.len());
    let successful_sub = &result.successful[0];
    assert_eq!(dest_value, successful_sub.matched_value.as_str().unwrap());

    // Verify the subscription works by changing the matching source
    let new_value = "updated_matching_value";
    value::set(ctx, (source_component_2, "/domain/SourceProp"), new_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that destination value updated from the correct source
    let updated_dest_value = value::get(ctx, (dest_component, "/domain/DestProp")).await?;
    assert_eq!(
        json!(new_value),
        updated_dest_value,
        "Destination should reflect change from matching source"
    );

    // Verify changing the non-matching source doesn't affect destination
    value::set(
        ctx,
        (source_component_1, "/domain/SourceProp"),
        "some_other_value",
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let final_dest_value = value::get(ctx, (dest_component, "/domain/DestProp")).await?;
    assert_eq!(
        json!(new_value),
        final_dest_value,
        "Destination should not change from non-matching source"
    );

    Ok(())
}

#[test]
async fn autosubscribe_mixed_explicit_and_implicit_suggestions(ctx: &mut DalContext) -> Result<()> {
    // Test component with some props having explicit suggestSources and others
    // relying on implicit suggestAsSourceFor from other components

    // Create source schemas - one with suggestAsSourceFor, one without
    create_simple_schema(ctx, "ExplicitSourceSchema", "ExplicitProp").await?;
    create_schema_with_suggest_as_source_for(
        ctx,
        "ImplicitSourceSchema",
        "ImplicitProp",
        "MixedDestSchema",
        "/domain/ImplicitDestProp",
    )
    .await?;

    // Create destination schema with mixed suggestion types
    let dest_schema_definition = r#"
        function main() {
            return {
                props: [
                    {
                        name: "ExplicitDestProp",
                        kind: "string",
                        suggestSources: [
                            { schema: "ExplicitSourceSchema", prop: "/domain/ExplicitProp" }
                        ]
                    },
                    {
                        name: "ImplicitDestProp",
                        kind: "string"
                        // No explicit suggestions - relies on ImplicitSourceSchema's suggestAsSourceFor
                    },
                    {
                        name: "NoDestProp",
                        kind: "string",
                        // no matches at all
                    },
                ]
            };
        }
    "#;
    dal_test::helpers::schema::variant::create(ctx, "MixedDestSchema", dest_schema_definition)
        .await?;

    // Create components
    let explicit_source = component::create(ctx, "ExplicitSourceSchema", "explicit_source").await?;
    let implicit_source = component::create(ctx, "ImplicitSourceSchema", "implicit_source").await?;
    let dest_component = component::create(ctx, "MixedDestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set matching values for both types
    let test_value_1 = "explicit_value";
    let test_value_2 = "implicit_value";

    value::set(ctx, (explicit_source, "/domain/ExplicitProp"), test_value_1).await?;
    value::set(
        ctx,
        (dest_component, "/domain/ExplicitDestProp"),
        test_value_1,
    )
    .await?;

    value::set(ctx, (implicit_source, "/domain/ImplicitProp"), test_value_2).await?;
    value::set(
        ctx,
        (dest_component, "/domain/ImplicitDestProp"),
        test_value_2,
    )
    .await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe and verify both types work correctly
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Should create 2 subscriptions - one explicit, one implicit
    assert_eq!(
        2,
        result.success_count(),
        "Should create subscriptions for both explicit and implicit suggestions"
    );
    assert!(!result.has_issues(), "Should not have any issues");

    // Verify the subscription details
    assert_eq!(2, result.successful.len());
    for successful_sub in &result.successful {
        // Each subscription should have matched its corresponding test value
        assert!(
            successful_sub.matched_value.as_str().unwrap() == test_value_1
                || successful_sub.matched_value.as_str().unwrap() == test_value_2
        );
    }

    // Verify both subscriptions work by changing source values
    let new_value_1 = "updated_explicit_value";
    let new_value_2 = "updated_implicit_value";

    value::set(ctx, (explicit_source, "/domain/ExplicitProp"), new_value_1).await?;
    value::set(ctx, (implicit_source, "/domain/ImplicitProp"), new_value_2).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that both destination values updated
    let dest_value_1 = value::get(ctx, (dest_component, "/domain/ExplicitDestProp")).await?;
    let dest_value_2 = value::get(ctx, (dest_component, "/domain/ImplicitDestProp")).await?;

    assert_eq!(
        json!(new_value_1),
        dest_value_1,
        "Explicit subscription should work"
    );
    assert_eq!(
        json!(new_value_2),
        dest_value_2,
        "Implicit subscription should work"
    );

    Ok(())
}

#[test]
async fn autosubscribe_no_suggestions_no_subscriptions(ctx: &mut DalContext) -> Result<()> {
    // Test that components without any prop suggestions don't create subscriptions

    // Create schemas without prop suggestions
    create_simple_schema(ctx, "SourceSchema", "SourceProp").await?;
    create_simple_schema(ctx, "DestSchema", "DestProp").await?;

    // Create components with matching values
    let source_component = component::create(ctx, "SourceSchema", "source").await?;
    let dest_component = component::create(ctx, "DestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let test_value = "matching_value";
    value::set(ctx, (source_component, "/domain/SourceProp"), test_value).await?;
    value::set(ctx, (dest_component, "/domain/DestProp"), test_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe and verify no subscriptions created
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert_eq!(
        0,
        result.success_count(),
        "Should not create subscriptions without suggestions"
    );

    assert!(!result.has_issues(), "Should not have any issues");

    // Verify destination value remains unchanged
    let dest_value = value::get(ctx, (dest_component, "/domain/DestProp")).await?;
    assert_eq!(
        json!(test_value),
        dest_value,
        "Destination value should remain unchanged"
    );

    Ok(())
}

#[test]
async fn autosubscribe_schema_mismatch(ctx: &mut DalContext) -> Result<()> {
    // Test that suggestions pointing to non-existent schemas/wrong schemas
    // don't create subscriptions

    // Create source schema
    create_simple_schema(ctx, "ActualSourceSchema", "SourceProp").await?;

    // Create destination schema with suggestions pointing to non-existent schema
    create_schema_with_suggest_sources(
        ctx,
        "DestSchema",
        "DestProp",
        "NonExistentSchema", // This schema doesn't exist
        "/domain/SourceProp",
    )
    .await?;

    // Create components
    let source_component = component::create(ctx, "ActualSourceSchema", "source").await?;
    let dest_component = component::create(ctx, "DestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set matching values
    let test_value = "matching_value";
    value::set(ctx, (source_component, "/domain/SourceProp"), test_value).await?;
    value::set(ctx, (dest_component, "/domain/DestProp"), test_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe and verify no subscriptions created
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert_eq!(
        0,
        result.success_count(),
        "Should not create subscriptions with non-existent schema"
    );

    assert!(!result.has_issues(), "Should not have any issues");

    // Verify destination value remains unchanged
    let dest_value = value::get(ctx, (dest_component, "/domain/DestProp")).await?;
    assert_eq!(
        json!(test_value),
        dest_value,
        "Destination value should remain unchanged"
    );

    Ok(())
}

#[test]
async fn autosubscribe_prop_path_mismatch(ctx: &mut DalContext) -> Result<()> {
    // Test that suggestions pointing to non-existent prop paths don't create subscriptions

    // Create source schema with actual prop
    create_simple_schema(ctx, "SourceSchema", "ActualProp").await?;

    // Create destination schema with suggestions pointing to non-existent prop path
    create_schema_with_suggest_sources(
        ctx,
        "DestSchema",
        "DestProp",
        "SourceSchema",
        "/domain/NonExistentProp", // This prop path doesn't exist
    )
    .await?;

    // Create components
    let source_component = component::create(ctx, "SourceSchema", "source").await?;
    let dest_component = component::create(ctx, "DestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set values
    let test_value = "matching_value";
    value::set(ctx, (source_component, "/domain/ActualProp"), test_value).await?;
    value::set(ctx, (dest_component, "/domain/DestProp"), test_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe and verify no subscriptions created
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert_eq!(
        0,
        result.success_count(),
        "Should not create subscriptions with non-existent prop path"
    );

    assert!(!result.has_issues(), "Should not have any issues");

    // Verify destination value remains unchanged
    let dest_value = value::get(ctx, (dest_component, "/domain/DestProp")).await?;
    assert_eq!(
        json!(test_value),
        dest_value,
        "Destination value should remain unchanged"
    );

    Ok(())
}

#[test]
async fn autosubscribe_array_values(ctx: &mut DalContext) -> Result<()> {
    // Test autosubscribe behavior with array values

    // Create source schema with array prop
    let source_schema_definition = r#"
        function main() {
            return {
                props: [
                    {
                        name: "ArrayProp",
                        kind: "array",
                        entry: {
                            name: "ArrayItem",
                            kind: "string"
                        }
                    }
                ]
            };
        }
    "#;
    dal_test::helpers::schema::variant::create(ctx, "ArraySourceSchema", source_schema_definition)
        .await?;

    // Create destination schema with array prop that suggests the source
    let dest_schema_definition = r#"
        function main() {
            return {
                props: [
                    {
                        name: "DestArrayProp",
                        kind: "array",
                        entry: {
                            name: "ArrayItem",
                            kind: "string"
                        },
                        suggestSources: [
                            { schema: "ArraySourceSchema", prop: "/domain/ArrayProp" }
                        ]
                    }
                ]
            };
        }
    "#;
    dal_test::helpers::schema::variant::create(ctx, "ArrayDestSchema", dest_schema_definition)
        .await?;

    // Create components
    let source_component = component::create(ctx, "ArraySourceSchema", "source").await?;
    let dest_component = component::create(ctx, "ArrayDestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set matching array values
    let test_array = json!(["item1", "item2", "item3"]);
    value::set(
        ctx,
        (source_component, "/domain/ArrayProp"),
        test_array.clone(),
    )
    .await?;
    value::set(
        ctx,
        (dest_component, "/domain/DestArrayProp"),
        test_array.clone(),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe and verify subscriptions work with arrays
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert_eq!(
        1,
        result.success_count(),
        "Should create exactly one subscription for array"
    );

    assert!(!result.has_issues(), "Should not have any issues");

    // Verify the subscription details
    assert_eq!(1, result.successful.len());
    let successful_sub = &result.successful[0];
    assert_eq!(test_array, successful_sub.matched_value);

    // Verify the subscription works by changing source array
    let new_array = json!(["new_item1", "new_item2"]);
    value::set(
        ctx,
        (source_component, "/domain/ArrayProp"),
        new_array.clone(),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that destination array updated
    let dest_value = value::get(ctx, (dest_component, "/domain/DestArrayProp")).await?;
    assert_eq!(
        new_array, dest_value,
        "Destination array should reflect source change"
    );

    Ok(())
}

#[test]
async fn autosubscribe_object_values(ctx: &mut DalContext) -> Result<()> {
    // Test autosubscribe behavior with complex object values

    // Create source schema with object prop
    let source_schema_definition = r#"
        function main() {
            return {
                props: [
                    {
                        name: "ObjectProp",
                        kind: "object",
                        children: [
                            {
                                name: "name",
                                kind: "string"
                            },
                            {
                                name: "age",
                                kind: "integer"
                            },
                            {
                                name: "active",
                                kind: "boolean"
                            }
                        ]
                    }
                ]
            };
        }
    "#;
    dal_test::helpers::schema::variant::create(ctx, "ObjectSourceSchema", source_schema_definition)
        .await?;

    // Create destination schema with object prop that suggests the source
    let dest_schema_definition = r#"
        function main() {
            return {
                props: [
                    {
                        name: "DestObjectProp",
                        kind: "object",
                        children: [
                            {
                                name: "name",
                                kind: "string"
                            },
                            {
                                name: "age",
                                kind: "integer"
                            },
                            {
                                name: "active",
                                kind: "boolean"
                            }
                        ],
                        suggestSources: [
                            { schema: "ObjectSourceSchema", prop: "/domain/ObjectProp" }
                        ]
                    }
                ]
            };
        }
    "#;
    dal_test::helpers::schema::variant::create(ctx, "ObjectDestSchema", dest_schema_definition)
        .await?;

    // Create components
    let source_component = component::create(ctx, "ObjectSourceSchema", "source").await?;
    let dest_component = component::create(ctx, "ObjectDestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set matching object values
    let test_object = json!({
        "name": "John Doe",
        "age": 30,
        "active": true
    });
    value::set(
        ctx,
        (source_component, "/domain/ObjectProp"),
        test_object.clone(),
    )
    .await?;
    value::set(
        ctx,
        (dest_component, "/domain/DestObjectProp"),
        test_object.clone(),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe and verify subscriptions work with objects
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    assert_eq!(
        1,
        result.success_count(),
        "Should create exactly one subscription for object"
    );

    assert!(!result.has_issues(), "Should not have any issues");

    // Verify the subscription details
    assert_eq!(1, result.successful.len());
    let successful_sub = &result.successful[0];
    assert_eq!(test_object, successful_sub.matched_value);

    // Verify the subscription works by changing source object
    let new_object = json!({
        "name": "Jane Smith",
        "age": 25,
        "active": false
    });
    value::set(
        ctx,
        (source_component, "/domain/ObjectProp"),
        new_object.clone(),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that destination object updated
    let dest_value = value::get(ctx, (dest_component, "/domain/DestObjectProp")).await?;
    assert_eq!(
        new_object, dest_value,
        "Destination object should reflect source change"
    );

    Ok(())
}

#[test]
async fn autosubscribe_multiple_props_same_component(ctx: &mut DalContext) -> Result<()> {
    // Test component with multiple props that could have subscriptions

    // Create source schemas
    create_simple_schema(ctx, "SourceSchemaA", "PropA").await?;
    create_simple_schema(ctx, "SourceSchemaB", "PropB").await?;

    // Create destination schema with multiple props having suggestions
    let dest_schema_definition = r#"
        function main() {
            return {
                props: [
                    {
                        name: "DestPropA",
                        kind: "string",
                        suggestSources: [
                            { schema: "SourceSchemaA", prop: "/domain/PropA" }
                        ]
                    },
                    {
                        name: "DestPropB",
                        kind: "string",
                        suggestSources: [
                            { schema: "SourceSchemaB", prop: "/domain/PropB" }
                        ]
                    }
                ]
            };
        }
    "#;
    dal_test::helpers::schema::variant::create(ctx, "MultiDestSchema", dest_schema_definition)
        .await?;

    // Create components
    let source_a = component::create(ctx, "SourceSchemaA", "source_a").await?;
    let source_b = component::create(ctx, "SourceSchemaB", "source_b").await?;
    let dest_component = component::create(ctx, "MultiDestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set matching values for both props
    let value_a = "value_a";
    let value_b = "value_b";

    value::set(ctx, (source_a, "/domain/PropA"), value_a).await?;
    value::set(ctx, (dest_component, "/domain/DestPropA"), value_a).await?;

    value::set(ctx, (source_b, "/domain/PropB"), value_b).await?;
    value::set(ctx, (dest_component, "/domain/DestPropB"), value_b).await?;

    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe and verify multiple subscriptions created correctly
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Should create 2 subscriptions
    assert_eq!(
        2,
        result.success_count(),
        "Should create subscriptions for both props"
    );

    assert!(!result.has_issues(), "Should not have any issues");

    // Verify the subscription details
    assert_eq!(2, result.successful.len());
    for successful_sub in &result.successful {
        // Each subscription should have matched its corresponding test value
        assert!(
            successful_sub.matched_value.as_str().unwrap() == value_a
                || successful_sub.matched_value.as_str().unwrap() == value_b
        );
    }

    // Verify both subscriptions work
    let new_value_a = "updated_value_a";
    let new_value_b = "updated_value_b";

    value::set(ctx, (source_a, "/domain/PropA"), new_value_a).await?;
    value::set(ctx, (source_b, "/domain/PropB"), new_value_b).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Check that both destination values updated
    let dest_value_a = value::get(ctx, (dest_component, "/domain/DestPropA")).await?;
    let dest_value_b = value::get(ctx, (dest_component, "/domain/DestPropB")).await?;

    assert_eq!(
        json!(new_value_a),
        dest_value_a,
        "First subscription should work"
    );
    assert_eq!(
        json!(new_value_b),
        dest_value_b,
        "Second subscription should work"
    );

    Ok(())
}

#[test]
async fn autosubscribe_between_two_components(ctx: &mut DalContext) -> Result<()> {
    // Test handling of circular suggestion relationships

    // Create schemas with circular suggestion relationships (A->B, B->A)
    let schema_a_definition = r#"
        function main() {
            return {
                props: [
                    {
                        name: "PropA",
                        kind: "string",
                        suggestSources: [
                            { schema: "SchemaB", prop: "/domain/PropB" }
                        ]
                    }
                ]
            };
        }
    "#;

    let schema_b_definition = r#"
        function main() {
            return {
                props: [
                    {
                        name: "PropB", 
                        kind: "string",
                        suggestSources: [
                            { schema: "SchemaA", prop: "/domain/PropA" }
                        ]
                    }
                ]
            };
        }
    "#;

    dal_test::helpers::schema::variant::create(ctx, "SchemaA", schema_a_definition).await?;
    dal_test::helpers::schema::variant::create(ctx, "SchemaB", schema_b_definition).await?;

    // Create components
    let component_a = component::create(ctx, "SchemaA", "comp_a").await?;
    let component_b = component::create(ctx, "SchemaB", "comp_b").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set values that would create circular dependencies if both subscriptions were created
    let test_value = "circular_value";
    value::set(ctx, (component_a, "/domain/PropA"), test_value).await?;
    value::set(ctx, (component_b, "/domain/PropB"), test_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Call autosubscribe on component A and verify it handles circular deps gracefully
    let result_a = Component::autosubscribe(ctx, component_a).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Should either create a subscription or detect a conflict, but not crash
    assert_eq!(
        result_a.success_count(),
        1,
        "Should handle circular suggestions gracefully"
    );

    // Call autosubscribe on component B
    let result_b = Component::autosubscribe(ctx, component_b).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
    // Should also handle gracefully
    assert_eq!(
        result_b.success_count(),
        1,
        "Should handle circular suggestions gracefully"
    );

    // Verify that we don't have infinite loops or crashes - the fact that we got here means success
    // The exact behavior (whether subscriptions are created or conflicts detected) is less important
    // than ensuring the function doesn't crash or loop infinitely
    Ok(())
}

#[test]
async fn autosubscribe_existing_subscription_skip(ctx: &mut DalContext) -> Result<()> {
    // Test that existing subscriptions are not overwritten or duplicated

    // Create schemas and components
    create_simple_schema(ctx, "SourceSchema", "SourceProp").await?;
    create_schema_with_suggest_sources(
        ctx,
        "DestSchema",
        "DestProp",
        "SourceSchema",
        "/domain/SourceProp",
    )
    .await?;

    let source_component = component::create(ctx, "SourceSchema", "source").await?;
    let dest_component = component::create(ctx, "DestSchema", "dest").await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Set matching values
    let test_value = "test_value";
    value::set(ctx, (source_component, "/domain/SourceProp"), test_value).await?;
    value::set(ctx, (dest_component, "/domain/DestProp"), test_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Manually create a subscription first
    value::subscribe(
        ctx,
        (dest_component, "/domain/DestProp"),
        (source_component, "/domain/SourceProp"),
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Verify the manual subscription works
    let initial_value = value::get(ctx, (dest_component, "/domain/DestProp")).await?;
    assert_eq!(
        json!(test_value),
        initial_value,
        "Manual subscription should work"
    );

    // Call autosubscribe and verify existing subscription is preserved (not duplicated or overwritten)
    let result = Component::autosubscribe(ctx, dest_component).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Should not create new subscriptions since one already exists
    assert_eq!(
        0,
        result.success_count(),
        "Should not create subscriptions when they already exist"
    );

    assert!(!result.has_issues(), "Should not have any issues");

    // Verify existing subscription still works
    let new_value = "updated_value";
    value::set(ctx, (source_component, "/domain/SourceProp"), new_value).await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    let final_value = value::get(ctx, (dest_component, "/domain/DestProp")).await?;
    assert_eq!(
        json!(new_value),
        final_value,
        "Existing subscription should still work after autosubscribe"
    );

    Ok(())
}

// Helper functions for tests

async fn create_schema_with_suggest_sources(
    ctx: &mut DalContext,
    schema_name: &str,
    prop_name: &str,
    suggested_schema: &str,
    suggested_prop: &str,
) -> Result<SchemaVariantId> {
    let variant_definition = format!(
        r#"
            function main() {{
                return {{
                    props: [
                        {{ 
                            name: "{prop_name}",
                            kind: "string",
                            suggestSources: [
                                {{ 
                                    schema: "{suggested_schema}",
                                    prop: "{suggested_prop}"
                                }}
                            ]
                        }},
                    ]
                }};
            }}
        "#,
    );

    let schema_variant_id =
        dal_test::helpers::schema::variant::create(ctx, schema_name, &variant_definition).await?;
    Ok(schema_variant_id)
}

async fn create_schema_with_suggest_as_source_for(
    ctx: &mut DalContext,
    schema_name: &str,
    prop_name: &str,
    target_schema: &str,
    target_prop: &str,
) -> Result<SchemaVariantId> {
    let variant_definition = format!(
        r#"
            function main() {{
                return {{
                    props: [
                        {{ 
                            name: "{prop_name}",
                            kind: "string",
                            suggestAsSourceFor: [
                                {{ 
                                    schema: "{target_schema}",
                                    prop: "{target_prop}"
                                }}
                            ]
                        }},
                    ]
                }};
            }}
        "#
    );

    let schema_variant_id =
        dal_test::helpers::schema::variant::create(ctx, schema_name, &variant_definition).await?;
    Ok(schema_variant_id)
}

async fn create_simple_schema(
    ctx: &mut DalContext,
    schema_name: &str,
    prop_name: &str,
) -> Result<SchemaVariantId> {
    let variant_definition = format!(
        r#"
            function main() {{
                return {{
                    props: [
                        {{ 
                            name: "{prop_name}",
                            kind: "string"
                        }},
                    ]
                }};
            }}
        "#,
    );

    let schema_variant_id =
        dal_test::helpers::schema::variant::create(ctx, schema_name, &variant_definition).await?;
    Ok(schema_variant_id)
}
