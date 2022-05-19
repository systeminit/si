use crate::dal::test;
use dal::component::ComponentKind;
use dal::edit_field::{EditFieldAble, Widget};
use dal::schema::RootProp;
use dal::socket::{Socket, SocketArity, SocketEdgeKind};
use dal::{DalContext, DalContextBuilder, JwtSecretKey, Prop, Schema, SchemaVariant};

use dal::test_harness::generate_fake_name;
use dal::{
    component::view::ComponentView,
    test_harness::{create_prop_of_kind_with_name, create_schema, create_schema_variant_with_root},
    AttributeContext, AttributeReadContext, AttributeValue, Component, PropKind, SchemaKind,
    StandardModel,
};
use pretty_assertions_sorted::assert_eq_sorted;

#[test]
async fn update_for_context_simple(ctx: &DalContext<'_, '_>) {
    // "name": String
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    let name_prop = create_prop_of_kind_with_name(ctx, PropKind::String, "name_prop").await;
    name_prop
        .set_parent_prop(ctx, root.domain_prop_id, base_attribute_read_context)
        .await
        .expect("cannot set parent of name_prop");

    let (component, _, _) =
        Component::new_for_schema_with_node(ctx, "Basic component", schema.id())
            .await
            .expect("Unable to create component");

    let read_context = AttributeReadContext {
        prop_id: None,
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        component_id: Some(*component.id()),
        ..AttributeReadContext::default()
    };
    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {},
            }
        ],
        component_view.properties,
    );

    let domain_value_id = *AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(root.domain_prop_id),
            component_id: Some(*component.id()),
            ..AttributeReadContext::any()
        },
    )
    .await
    .expect("cannot get domain AttributeValue")
    .pop()
    .expect("domain AttributeValue not found")
    .id();
    let base_name_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*name_prop.id()),
            component_id: Some(*component.id()),
            ..AttributeReadContext::any()
        },
    )
    .await
    .expect("cannot get name AttributeValue")
    .pop()
    .expect("name AttributeValue not found");

    let update_context = AttributeContext::builder()
        .set_prop_id(*name_prop.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id())
        .set_component_id(*component.id())
        .to_context()
        .expect("cannot build write AttributeContext");

    let (_, name_value_id, _) = AttributeValue::update_for_context(
        ctx,
        *base_name_value.id(),
        Some(domain_value_id),
        update_context,
        Some(serde_json::to_value("Miles".to_string()).expect("cannot create new Value")),
        None,
    )
    .await
    .expect("cannot set value for context");

    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {
                    "name_prop": "Miles",
                },
            }
        ],
        component_view.properties,
    );

    AttributeValue::update_for_context(
        ctx,
        name_value_id,
        Some(domain_value_id),
        update_context,
        Some(serde_json::to_value("Iria".to_string()).expect("cannot create new value")),
        None,
    )
    .await
    .expect("cannot update value for context");

    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");

    assert_eq_sorted!(
        serde_json::json![
            {
                "si": {
                    "name": "Basic component",
                },
                "domain": {
                    "name_prop": "Iria",
                },
            }
        ],
        component_view.properties,
    );
}

#[test]
async fn insert_for_context_simple(ctx: &DalContext<'_, '_>) {
    let mut schema = create_schema(ctx, &SchemaKind::Concrete).await;
    let (schema_variant, root) = create_schema_variant_with_root(ctx, *schema.id()).await;
    schema
        .set_default_schema_variant_id(ctx, Some(*schema_variant.id()))
        .await
        .expect("cannot set default schema variant");

    let base_attribute_read_context = AttributeReadContext {
        schema_id: Some(*schema.id()),
        schema_variant_id: Some(*schema_variant.id()),
        ..AttributeReadContext::default()
    };

    let array_prop = create_prop_of_kind_with_name(ctx, PropKind::Array, "array_prop").await;
    array_prop
        .set_parent_prop(ctx, root.domain_prop_id, base_attribute_read_context)
        .await
        .expect("cannot set parent of array_prop");

    let array_element = create_prop_of_kind_with_name(ctx, PropKind::String, "array_element").await;
    array_element
        .set_parent_prop(ctx, *array_prop.id(), base_attribute_read_context)
        .await
        .expect("cannot set parent of array_element");

    let (component, _, _) =
        Component::new_for_schema_with_node(ctx, "Array Component", schema.id())
            .await
            .expect("Unable to create component");

    let read_context = AttributeReadContext {
        prop_id: None,
        component_id: Some(*component.id()),
        ..base_attribute_read_context
    };

    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");
    assert_eq_sorted!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "domain": {},
        }],
        component_view.properties,
    );

    let array_value = AttributeValue::find_for_context(
        ctx,
        AttributeReadContext {
            prop_id: Some(*array_prop.id()),
            component_id: Some(*component.id()),
            ..base_attribute_read_context
        },
    )
    .await
    .expect("cannot get array AttributeValue")
    .pop()
    .expect("array AttributeValue not found");
    let update_context = AttributeContext::builder()
        .set_prop_id(*array_element.id())
        .set_schema_id(*schema.id())
        .set_schema_variant_id(*schema_variant.id())
        .set_component_id(*component.id())
        .to_context()
        .expect("cannot build write AttributeContext");

    let _new_array_element_value_id =
        AttributeValue::insert_for_context(ctx, update_context, *array_value.id(), None, None)
            .await
            .expect("cannot insert new array element");

    let component_view = ComponentView::for_context(ctx, read_context)
        .await
        .expect("cannot get component view");
    assert_eq_sorted!(
        serde_json::json![{
            "si": {
                "name": "Array Component",
            },
            "domain": {
                "array_prop": [],
            },
        }],
        component_view.properties,
    );
}

#[test]
async fn set_field_in_nested_object(
    dal_context_builder: DalContextBuilder,
    jwt_secret_key: &JwtSecretKey,
) {
    use local_helpers::*;

    let mut transactions_starter = dal_context_builder
        .transactions_starter()
        .await
        .expect("failed to build transactions starter");
    let transactions = transactions_starter
        .start()
        .await
        .expect("failed to start transactions");
    let (nba, _auth_token) = ::dal::test::helpers::billing_account_signup(
        &dal_context_builder,
        &transactions,
        jwt_secret_key,
    )
    .await;
    let application_id =
        ::dal::test::helpers::create_application(&dal_context_builder, &transactions, &nba).await;
    let application_id = {
        use dal::StandardModel;
        *application_id.id()
    };
    let default_dal_context = ::dal::test::helpers::create_ctx_for_new_change_set_and_edit_session(
        &dal_context_builder,
        &transactions,
        &nba,
        application_id,
    )
    .await;
    let ctx = &default_dal_context;

    // Create a schema for modeling bands:
    //
    // ```yaml
    // - band_name: string
    // - formed:
    //   - year: integer
    //   - country: string
    //   - city: string
    // ```
    let (schema, variant, root_prop) = create_schema_and_variant(ctx).await;
    add_props_to_domain(
        ctx,
        &schema,
        &variant,
        &root_prop,
        &[
            create_prop(ctx, "band_name", PropKind::String).await,
            create_prop_with_children(
                ctx,
                "formed",
                PropKind::Object,
                &schema,
                &variant,
                &[
                    create_prop(ctx, "year", PropKind::Integer).await,
                    create_prop(ctx, "country", PropKind::String).await,
                    create_prop(ctx, "city", PropKind::String).await,
                ],
            )
            .await,
        ],
    )
    .await;

    let component = create_component_from_variant(ctx, &variant).await;

    // Component view's domain is initially empty
    assert_eq_sorted!(
        serde_json::json!({}),
        component_domain_value(ctx, &schema, &variant, &component).await
    );

    // All edit fields should be present, but all with `None` values
    let domain_edit_fields = component_domain_edit_fields(ctx, &component).await;
    let band_name_edit_field = domain_edit_fields
        .iter()
        .find(|field| field.name == "band_name")
        .expect("failed to find edit field");
    assert!(band_name_edit_field.value().is_none());
    let formed_edit_field = domain_edit_fields
        .iter()
        .find(|field| field.name == "formed")
        .expect("failed to find edit field");
    assert!(formed_edit_field.value().is_none());
    match formed_edit_field.widget() {
        Widget::Header(header) => {
            let formed_edit_fields = header.edit_fields();
            assert!(formed_edit_fields
                .iter()
                .find(|field| field.name == "year")
                .expect("failed to find edit field for year")
                .value()
                .is_none());
            assert!(formed_edit_fields
                .iter()
                .find(|field| field.name == "country")
                .expect("failed to find edit field for country")
                .value()
                .is_none());
            assert!(formed_edit_fields
                .iter()
                .find(|field| field.name == "city")
                .expect("failed to find edit field city")
                .value()
                .is_none());
        }
        _ => panic!("wrong header type"),
    };

    // Set a value which is a direct child of the domain prop (i.e. no nesting)
    component
        .set_value_by_json_pointer(
            ctx,
            "/root/domain/band_name",
            Some(serde_json::json!("Pain of Salvation")),
        )
        .await
        .expect("failed to set value");

    // Component view's domain has one entry after having set a value
    assert_eq_sorted!(
        serde_json::json!({
            "band_name": "Pain of Salvation",
        }),
        component_domain_value(ctx, &schema, &variant, &component).await
    );

    // Now there should be one edit field that has a value, but all should still be present
    let domain_edit_fields = component_domain_edit_fields(ctx, &component).await;
    let band_name_edit_field = domain_edit_fields
        .iter()
        .find(|field| field.name == "band_name")
        .expect("failed to find edit field");
    assert_eq!(
        band_name_edit_field.value(),
        &Some(serde_json::json!("Pain of Salvation"))
    );
    let formed_edit_field = domain_edit_fields
        .iter()
        .find(|field| field.name == "formed")
        .expect("failed to find edit field");
    assert!(formed_edit_field.value().is_none());
    match formed_edit_field.widget() {
        Widget::Header(header) => {
            let formed_edit_fields = header.edit_fields();
            assert!(formed_edit_fields
                .iter()
                .find(|field| field.name == "year")
                .expect("failed to find edit field for year")
                .value()
                .is_none());
            assert!(formed_edit_fields
                .iter()
                .find(|field| field.name == "country")
                .expect("failed to find edit field for country")
                .value()
                .is_none());
            assert!(formed_edit_fields
                .iter()
                .find(|field| field.name == "city")
                .expect("failed to find edit field for city")
                .value()
                .is_none());
        }
        _ => panic!("wrong header type"),
    };

    // Now we'll set a couple of values under an object (i.e. nested), but not every single value
    // so there are still unset values
    component
        .set_value_by_json_pointer(
            ctx,
            "/root/domain/formed/year",
            Some(serde_json::json!(1984)),
        )
        .await
        .expect("failed to set value");
    component
        .set_value_by_json_pointer(
            ctx,
            "/root/domain/formed/country",
            Some(serde_json::json!("Sweden")),
        )
        .await
        .expect("failed to set value");

    // transactions.commit().await.expect("no surprises please");
    // panic!("AHHH");

    // The nested object is created and partially populated when sub-fields are set
    assert_eq_sorted!(
        serde_json::json!({
            "band_name": "Pain of Salvation",
            "formed": {
                "country": "Sweden",
                "year": 1984,
            },
        }),
        component_domain_value(ctx, &schema, &variant, &component).await
    );

    // The nested object should have the values set, but the remaining edit fields that were not
    // set should still be present
    let domain_edit_fields = component_domain_edit_fields(ctx, &component).await;
    let band_name_edit_field = domain_edit_fields
        .iter()
        .find(|field| field.name == "band_name")
        .expect("failed to find edit field");
    assert_eq!(
        band_name_edit_field.value(),
        &Some(serde_json::json!("Pain of Salvation"))
    );
    let formed_edit_field = domain_edit_fields
        .iter()
        .find(|field| field.name == "formed")
        .expect("failed to find edit field");
    assert_eq!(formed_edit_field.value(), &Some(serde_json::json!({})));
    match formed_edit_field.widget() {
        Widget::Header(header) => {
            let formed_edit_fields = header.edit_fields();
            assert_eq!(
                formed_edit_fields
                    .iter()
                    .find(|field| field.name == "year")
                    .expect("failed to find edit field for year")
                    .value(),
                &Some(serde_json::json!(1984))
            );
            assert_eq!(
                formed_edit_fields
                    .iter()
                    .find(|field| field.name == "country")
                    .expect("failed to find edit field for country")
                    .value(),
                &Some(serde_json::json!("Sweden"))
            );
            assert!(formed_edit_fields
                .iter()
                .find(|field| field.name == "city")
                .expect("failed to find edit field for city")
                .value()
                .is_none());
        }
        _ => panic!("wrong header type"),
    };
}

mod local_helpers {
    use dal::edit_field::{EditField, Widget};

    use super::*;

    pub async fn component_domain_edit_fields(
        ctx: &DalContext<'_, '_>,
        component: &Component,
    ) -> Vec<EditField> {
        let mut edit_fields = Component::get_edit_fields(ctx, component.id())
            .await
            .expect("failed to get edit fields");
        assert_eq!(
            1,
            edit_fields.len(),
            "edit fields vec must contain one element"
        );
        let root_edit_field = edit_fields.pop().unwrap();
        assert_eq!(
            "root", &root_edit_field.name,
            "root edit field must be named 'root'"
        );
        let domain_edit_fields = match root_edit_field.into_widget() {
            Widget::Header(header) => {
                let mut header_fields = header.into_edit_fields();
                let domain_idx = header_fields
                    .iter()
                    .position(|field| field.name == "domain")
                    .expect("failed to find domain edit field");
                let domain_edit_field = header_fields.remove(domain_idx);

                match domain_edit_field.into_widget() {
                    Widget::Header(header) => header.into_edit_fields(),
                    _ => panic!("domain widget must be type header"),
                }
            }
            _ => panic!("root widget must be type header"),
        };
        domain_edit_fields
    }

    pub async fn create_schema_and_variant(
        ctx: &DalContext<'_, '_>,
    ) -> (Schema, SchemaVariant, RootProp) {
        let schema_name = generate_fake_name();
        let mut schema = Schema::new(
            ctx,
            &schema_name,
            &SchemaKind::Concrete,
            &ComponentKind::Standard,
        )
        .await
        .expect("cannot create schema");

        let schema_variant_name = generate_fake_name();
        let (variant, variant_root_prop) =
            SchemaVariant::new(ctx, *schema.id(), schema_variant_name)
                .await
                .expect("cannot create schema variant");

        schema
            .set_default_schema_variant_id(ctx, Some(*variant.id()))
            .await
            .expect("cannot set default variant");

        let includes_socket = Socket::new(
            ctx,
            "includes",
            &SocketEdgeKind::Includes,
            &SocketArity::Many,
            &schema.kind().into(),
        )
        .await
        .expect("cannot create includes socket");
        variant
            .add_socket(ctx, includes_socket.id())
            .await
            .expect("cannot add socket to variant");

        (schema, variant, variant_root_prop)
    }

    pub async fn add_props_to_domain(
        ctx: &DalContext<'_, '_>,
        schema: &Schema,
        variant: &SchemaVariant,
        root_prop: &RootProp,
        children: &[Prop],
    ) {
        let base_attribute_read_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*variant.id()),
            ..AttributeReadContext::default()
        };

        for child in children {
            child
                .set_parent_prop(ctx, root_prop.domain_prop_id, base_attribute_read_context)
                .await
                .expect("cannot set parent prop for domain prop");
        }
    }

    pub async fn add_props_to_prop(
        ctx: &DalContext<'_, '_>,
        schema: &Schema,
        variant: &SchemaVariant,
        prop: &Prop,
        children: &[Prop],
    ) {
        let base_attribute_read_context = AttributeReadContext {
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*variant.id()),
            ..AttributeReadContext::default()
        };

        for child in children {
            child
                .set_parent_prop(ctx, *prop.id(), base_attribute_read_context)
                .await
                .expect("cannot set parent prop for child prop");
        }
    }

    pub async fn create_component_from_variant(
        ctx: &DalContext<'_, '_>,
        variant: &SchemaVariant,
    ) -> Component {
        let name = generate_fake_name();
        let (component, _, _) =
            Component::new_for_schema_variant_with_node(ctx, name, variant.id())
                .await
                .expect("cannot create component");
        component
    }

    pub async fn create_prop(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        kind: PropKind,
    ) -> Prop {
        Prop::new(ctx, name, kind).await.expect("cannot crate prop")
    }

    pub async fn create_prop_with_children(
        ctx: &DalContext<'_, '_>,
        name: impl AsRef<str>,
        kind: PropKind,
        schema: &Schema,
        variant: &SchemaVariant,
        children: &[Prop],
    ) -> Prop {
        let prop = Prop::new(ctx, name, kind).await.expect("cannot crate prop");
        add_props_to_prop(ctx, schema, variant, &prop, children).await;
        prop
    }

    pub async fn component_view_for_component(
        ctx: &DalContext<'_, '_>,
        schema: &Schema,
        variant: &SchemaVariant,
        component: &Component,
    ) -> ComponentView {
        let read_context = AttributeReadContext {
            prop_id: None,
            schema_id: Some(*schema.id()),
            schema_variant_id: Some(*variant.id()),
            component_id: Some(*component.id()),
            ..AttributeReadContext::default()
        };
        ComponentView::for_context(ctx, read_context)
            .await
            .expect("cannot get component view for context")
    }

    pub async fn component_domain_value(
        ctx: &DalContext<'_, '_>,
        schema: &Schema,
        variant: &SchemaVariant,
        component: &Component,
    ) -> serde_json::Value {
        component_view_for_component(ctx, schema, variant, component)
            .await
            .properties
            .get("domain")
            .expect("failed to find 'domain' at root of component view")
            .clone()
    }
}
