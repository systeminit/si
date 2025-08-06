use dal::{
    AttributeValue,
    AttributeValueId,
    Component,
    DalContext,
    Prop,
    SchemaVariantId,
    attribute::{
        attributes::{
            AttributeValueIdent,
            Source,
        },
        value::default_subscription::{
            DefaultSubscription,
            detect_possible_default_connections,
        },
    },
    diagram::view::View,
    prop::PropPath,
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
    let (_, _, default_dest_a_variant_id, _) = variants_with_prop_suggestions(ctx).await?;

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

    let sources = AttributeValue::get_default_subscription_sources(ctx).await?;

    let mut sources_as_av_ids: Vec<AttributeValueId> =
        sources.iter().map(|source| source.av_id).collect();
    sources_as_av_ids.sort();
    default_source_av_ids.sort();

    assert_eq!(
        sources_as_av_ids, default_source_av_ids,
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

    let mut possible_defaults = detect_possible_default_connections(ctx, dest_component_b_id)
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

    let possible_defaults_two = detect_possible_default_connections(ctx, dest_component_b_id)
        .await
        .expect("should be able to detect possible default connections");

    // one of the suggestions is now ambiguous since it could be either of the two source components
    assert_eq!(possible_defaults_two.len(), 2);

    let source_object_a_av_id = Component::attribute_values_for_prop_by_id(
        ctx,
        source_component_a_id,
        &["root", "domain", "source_object_a"],
    )
    .await
    .expect("should be able to find avs for prop")
    .pop()
    .expect("should have a an av id");

    AttributeValue::set_as_default_subscription_source(ctx, source_object_a_av_id)
        .await
        .expect("should be able to set as default subscription source");

    let frank_drebin = Component::new(
        ctx,
        "frank drebin, jr.",
        default_dest_a_variant_id,
        View::get_id_for_default(ctx).await?,
    )
    .await?;

    let sources = Component::sources(ctx, frank_drebin.id()).await?;
    let expected_sources = vec![
        (
            AttributeValueIdent::new("/si/name"),
            Source::Value("frank drebin, jr.".into()),
        ),
        (
            AttributeValueIdent::new("/si/type"),
            Source::Value("component".into()),
        ),
        (
            AttributeValueIdent::new("/domain/dest_string"),
            Source::Subscription {
                component: source_component_a_id.into(),
                path: "/domain/source_string".into(),
                keep_existing_subscriptions: None,
                func: None,
            },
        ),
        (
            AttributeValueIdent::new("/domain/dest_array_of_object_1/0"),
            Source::Subscription {
                component: source_component_a_id.into(),
                path: "/domain/source_object_a".into(),
                keep_existing_subscriptions: None,
                func: None,
            },
        ),
        (
            AttributeValueIdent::new("/domain/dest_integer"),
            Source::Subscription {
                component: source_component_a_id.into(),
                path: "/domain/source_integer".into(),
                keep_existing_subscriptions: None,
                func: None,
            },
        ),
    ];

    assert_eq!(expected_sources, sources);

    Ok(())
}

#[test]
async fn test_is_same_type_as(ctx: &DalContext) -> Result<()> {
    variants_with_prop_suggestions(ctx).await?;

    let default_source_a_id = variant::id(ctx, "default_source_a")
        .await
        .expect("should be able to find variant id");
    let default_source_b_id = variant::id(ctx, "default_source_b")
        .await
        .expect("should be able to find variant id");

    let source_object_a_from_a = Prop::find_prop_by_path(
        ctx,
        default_source_a_id,
        &PropPath::new(["root", "domain", "source_object_a"]),
    )
    .await
    .expect("should be able to find prop");

    let source_object_a_from_b = Prop::find_prop_by_path(
        ctx,
        default_source_b_id,
        &PropPath::new(["root", "domain", "source_object_a"]),
    )
    .await
    .expect("should be able to find prop");

    assert!(
        source_object_a_from_a
            .is_same_type_as(ctx, &source_object_a_from_b)
            .await
            .expect("should be able to compare types")
    );

    let source_object_b = Prop::find_prop_by_path(
        ctx,
        default_source_a_id,
        &PropPath::new(["root", "domain", "source_object_b"]),
    )
    .await
    .expect("should be able to find prop");

    assert_eq!(
        false,
        source_object_a_from_a
            .is_same_type_as(ctx, &source_object_b)
            .await
            .expect("should be able to compare types"),
    );
    assert_eq!(
        false,
        source_object_a_from_b
            .is_same_type_as(ctx, &source_object_b)
            .await
            .expect("should be able to compare types"),
    );

    let source_array_of_string = Prop::find_prop_by_path(
        ctx,
        default_source_a_id,
        &PropPath::new(["root", "domain", "source_array_of_string"]),
    )
    .await
    .expect("should be able to find prop");

    let source_array_of_integer = Prop::find_prop_by_path(
        ctx,
        default_source_a_id,
        &PropPath::new(["root", "domain", "source_array_of_integer"]),
    )
    .await
    .expect("should be able to find prop");

    assert_eq!(
        false,
        source_array_of_integer
            .is_same_type_as(ctx, &source_array_of_string)
            .await
            .expect("should be able to compare types"),
    );
    assert_eq!(
        false,
        source_array_of_string
            .is_same_type_as(ctx, &source_array_of_integer)
            .await
            .expect("should be able to compare types"),
    );

    let source_array_of_object_1_from_a = Prop::find_prop_by_path(
        ctx,
        default_source_a_id,
        &PropPath::new(["root", "domain", "source_array_of_object_1"]),
    )
    .await
    .expect("should be able to find prop");

    let source_array_of_object_2 = Prop::find_prop_by_path(
        ctx,
        default_source_a_id,
        &PropPath::new(["root", "domain", "source_array_of_object_2"]),
    )
    .await
    .expect("should be able to find prop");
    let source_array_of_object_1_from_b = Prop::find_prop_by_path(
        ctx,
        default_source_b_id,
        &PropPath::new(["root", "domain", "source_array_of_object_1"]),
    )
    .await
    .expect("should be able to find prop");

    assert!(
        source_array_of_object_1_from_a
            .is_same_type_as(ctx, &source_array_of_object_1_from_b)
            .await
            .expect("should be able to compare types"),
    );

    assert_eq!(
        false,
        source_array_of_object_1_from_a
            .is_same_type_as(ctx, &source_array_of_object_2)
            .await
            .expect("should be able to compare types"),
    );

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
                name: "source_object_a",
                kind: "object",
                children: [
                    {
                        name: "child_bool",
                        kind: "boolean",
                    },
                    {
                        name: "child_string",
                        kind: "string",
                    },
                ],
                suggestAsSourceFor: [
                    { schema: "default_destination_a", prop: "/domain/dest_object_a" },
                ]
            },
            {
                name: "source_object_b",
                kind: "object",
                children: [
                    {
                        name: "child_integer",
                        kind: "integer",
                    },
                    {
                        name: "child_string",
                        kind: "string",
                    },
                ],
                suggestAsSourceFor: [
                    { schema: "default_destination_a", prop: "/domain/dest_object_a" },
                ]
            },
            {
                name: "source_object_b",
                kind: "object",
                children: [
                    {
                        name: "child_integer",
                        kind: "integer",
                    },
                    {
                        name: "child_string",
                        kind: "string",
                    },
                ],
                suggestAsSourceFor: [
                    { schema: "default_destination_a", prop: "/domain/dest_object_a" },
                ]
            },
            {
                name: "source_array_of_string",
                kind: "array",
                entry: {
                    name: "child_string",
                    kind: "string",
                },
            },
            {
                name: "source_array_of_object_1",
                kind: "array",
                entry: {
                    name: "child_object",
                    kind: "object",
                    children: [
                        {
                            name: "child_string",
                            kind: "string",
                        },
                        {
                            name: "child_integer",
                            kind: "integer",
                        },
                    ],
                },
            },
            {
                name: "source_array_of_object_2",
                kind: "array",
                entry: {
                    name: "child_object",
                    kind: "object",
                    children: [
                        {
                            name: "child_bool",
                            kind: "boolean",
                        },
                        {
                            name: "child_integer",
                            kind: "integer",
                        },
                    ],
                },
            },
            {
                name: "source_array_of_integer",
                kind: "array",
                entry: {
                    name: "child_integer",
                    kind: "integer",
                },
            },
            {
                name: "source_map_of_string",
                kind: "map",
                entry: {
                    name: "child_string",
                    kind: "string",
                },
            },
            {
                name: "source_map_of_bool",
                kind: "map",
                entry: {
                    name: "child_bool",
                    kind: "boolean",
                },
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
                name: "source_object_a",
                kind: "object",
                children: [
                    {
                        name: "child_bool",
                        kind: "boolean",
                    },
                    {
                        name: "child_string",
                        kind: "string",
                    },
                ],
                suggestAsSourceFor: [
                    { schema: "default_destination_a", prop: "/domain/dest_object_a" },
                ]
            },
            {
                name: "source_array_of_object_1",
                kind: "array",
                entry: {
                    name: "child_object",
                    kind: "object",
                    children: [
                        {
                            name: "child_string",
                            kind: "string",
                        },
                        {
                            name: "child_integer",
                            kind: "integer",
                        },
                    ],
                },
            },
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
                    name: "dest_object_a",
                    kind: "object",
                    children: [
                        {
                            name: "child_integer",
                            kind: "integer",
                        },
                        {
                            name: "child_string",
                            kind: "string",
                        },
                    ],
                    suggestSources: [
                        { schema: "default_source_a", prop: "/domain/source_object_a" },
                    ]
                },
                {
                    name: "dest_array_of_object_1",
                    kind: "array",
                    entry: {
                        name: "child_object",
                        kind: "object",
                        children: [
                            {
                                name: "child_bool",
                                kind: "boolean",
                            },
                            {
                                name: "child_string",
                                kind: "string",
                            },
                        ],
                        suggestSources: [
                            { schema: "default_source_a", prop: "/domain/source_object_a" },
                        ]
                    }
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
