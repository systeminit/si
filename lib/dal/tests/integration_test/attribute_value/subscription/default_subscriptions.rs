use dal::{
    AttributeValue,
    Component,
    DalContext,
    SchemaVariantId,
    attribute::value::default_subscription::{
        DefaultSubscription,
        calculate_all_prop_suggestions_for_change_set,
        detect_possible_default_connections,
    },
};
use dal_test::{
    Result,
    helpers::{
        component,
        schema::variant,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;

#[test]
async fn test_set_as_default_subscription_source(ctx: &DalContext) -> Result<()> {
    variants_with_prop_suggestions(ctx).await?;

    let suggestions_for_change_set = calculate_all_prop_suggestions_for_change_set(ctx).await?;

    let source_component_a_id = component::create(ctx, "default_source_a", "default_source_a")
        .await
        .expect("should be able to create component");
    let source_component_b_id = component::create(ctx, "default_source_b", "default_source_b")
        .await
        .expect("should be able to create component");

    let _dest_component_a_id =
        component::create(ctx, "default_destination_a", "default_destination_a")
            .await
            .expect("should be able to create component");

    assert!(
        AttributeValue::get_default_subscription_sources(ctx)
            .await?
            .is_empty()
    );

    let default_sources_paths = [
        ["root", "domain", "source_bool"],
        ["root", "domain", "source_string"],
        ["root", "domain", "source_integer"],
    ];

    let mut default_source_av_ids = vec![];

    for path in &default_sources_paths {
        let av_id = Component::attribute_values_for_prop_by_id(ctx, source_component_a_id, path)
            .await
            .expect("should be able to find avs for prop")
            .pop()
            .expect("should have a prop");

        AttributeValue::set_as_default_subscription_source(ctx, av_id)
            .await
            .expect("should be able to set as default subscription source");

        default_source_av_ids.push(av_id);
    }

    let mut sources = AttributeValue::get_default_subscription_sources(ctx).await?;

    sources.sort();
    default_source_av_ids.sort();

    assert_eq!(
        default_source_av_ids, sources,
        "default source av ids should match"
    );

    let dest_component_b_id =
        component::create(ctx, "default_destination_a", "default_destination_a")
            .await
            .expect("should be able to create component");

    let default_destinations_paths = [
        ["root", "domain", "dest_bool"],
        ["root", "domain", "dest_string"],
        ["root", "domain", "dest_integer"],
    ];

    let mut expected_defaults = vec![];
    for (idx, path) in default_destinations_paths.iter().enumerate() {
        let dest_av_id = Component::attribute_values_for_prop_by_id(ctx, dest_component_b_id, path)
            .await
            .expect("should be able to find avs for prop")
            .pop()
            .expect("should have a prop");

        let source_av_id = default_source_av_ids
            .get(idx)
            .copied()
            .expect("should have a matching source");

        expected_defaults.push(DefaultSubscription {
            source_av_id,
            dest_av_id,
        });
    }

    let mut possible_defaults =
        detect_possible_default_connections(ctx, dest_component_b_id, &suggestions_for_change_set)
            .await
            .expect("should be able to detect possible default connections");

    possible_defaults.sort();
    expected_defaults.sort();

    assert_eq!(expected_defaults, possible_defaults);

    for default_sub in possible_defaults {
        default_sub
            .subscribe(ctx)
            .await
            .unwrap_or_else(|_| panic!("should be able to subscribe: {default_sub:?}"));
    }

    let conflicting_av_id = Component::attribute_values_for_prop_by_id(
        ctx,
        source_component_b_id,
        &["root", "domain", "source_bool"],
    )
    .await
    .expect("should be able to find avs for prop")
    .pop()
    .expect("should have a prop");

    AttributeValue::set_as_default_subscription_source(ctx, conflicting_av_id)
        .await
        .expect("should be able to set as default subscription source");

    let possible_defaults_two =
        detect_possible_default_connections(ctx, dest_component_b_id, &suggestions_for_change_set)
            .await
            .expect("should be able to detect possible default connections");

    // one of the suggestions is now ambiguous since it could be either of the two source components
    assert_eq!(possible_defaults_two.len(), 2);

    Ok(())
}

async fn variants_with_prop_suggestions(
    ctx: &DalContext,
) -> Result<(
    SchemaVariantId,
    SchemaVariantId,
    SchemaVariantId,
    SchemaVariantId,
)> {
    let default_source_a_id = variant::create(
        ctx,
        "default_source_a",
        r#"
        function main() {
        return {
            props: [
            {
                name: "source_bool",
                kind: "boolean",
                suggestAsSourceFor: [
                    { schema: "default_destination_a", prop: "/domain/dest_boolean" },
                ]
            },
            {
                name: "source_string",
                kind: "string",
                suggestAsSourceFor: [
                    { schema: "default_destination_a", prop: "/domain/dest_string" },
                    { schema: "default_destination_b", prop: "/domain/dest_string" },
                ]
            },
            {
                name: "source_integer",
                kind: "integer",
                suggestAsSourceFor: [
                    { schema: "default_destination_a", prop: "/domain/dest_integer" },
                    { schema: "default_destination_b", prop: "/domain/dest_integer" },
                ]
            }
            ]
        }}"#,
    )
    .await?;

    let default_source_b_id = variant::create(
        ctx,
        "default_source_b",
        r#"
        function main() {
        return {
            props: [
            {
                name: "source_bool",
                kind: "boolean",
                suggestAsSourceFor: [
                    { schema: "default_destination_a", prop: "/domain/dest_bool" },
                ]
            },
            {
                name: "source_string",
                kind: "string",
                suggestAsSourceFor: [
                    { schema: "default_destination_a", prop: "/domain/dest_string" },
                    { schema: "default_destination_b", prop: "/domain/dest_string" },
                ]
            },
            {
                name: "source_integer",
                kind: "integer",
                suggestAsSourceFor: [
                    { schema: "default_destination_a", prop: "/domain/dest_integer" },
                    { schema: "default_destination_b", prop: "/domain/dest_integer" },
                ]
            }
            ]
        }}"#,
    )
    .await?;

    let default_destination_a_id = variant::create(
        ctx,
        "default_destination_a",
        r#"
        function main() {
        return {
            props: [
                {
                    name: "dest_string",
                    kind: "string",
                    suggestSources: [
                        { schema: "default_source_a", prop: "/domain/source_string" },
                        { schema: "default_source_b", prop: "/domain/source_string" },
                    ]
                },
                {
                    name: "dest_integer",
                    kind: "integer",
                    suggestSources: [
                        { schema: "default_source_a", prop: "/domain/source_integer" },
                        { schema: "default_source_b", prop: "/domain/source_integer" },
                    ]
                },
                {
                    name: "dest_bool",
                    kind: "boolean",
                    suggestSources: [
                        { schema: "default_source_a", prop: "/domain/source_bool" },
                        { schema: "default_source_b", prop: "/domain/source_bool" },
                    ]
                }
            ]
        }}"#,
    )
    .await?;

    let default_destination_b_id = variant::create(
        ctx,
        "default_destination_b",
        r#"
        function main() {
        return {
            props: [
                {
                    name: "dest_string",
                    kind: "string",
                    suggestSources: [
                        { schema: "default_source_a", prop: "/domain/source_string" },
                        { schema: "default_source_b", prop: "/domain/source_string" },
                    ]
                },
                {
                    name: "dest_integer",
                    kind: "integer",
                    suggestSources: [
                        { schema: "default_source_a", prop: "/domain/source_integer" },
                        { schema: "default_source_b", prop: "/domain/source_integer" },
                    ]
                },
                {
                    name: "dest_boolean",
                    kind: "boolean",
                }
            ],
        }}"#,
    )
    .await?;

    Ok((
        default_source_a_id,
        default_source_b_id,
        default_destination_a_id,
        default_destination_b_id,
    ))
}
