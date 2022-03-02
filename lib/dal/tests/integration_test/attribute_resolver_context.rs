use dal::{
    attribute_resolver_context::{AttributeResolverContext, AttributeResolverContextBuilder},
    ComponentId, PropId, SchemaId, SchemaVariantId, SystemId,
};

use pretty_assertions_sorted::assert_eq;

// NOTE(nick): there are only error permutations tests for fields that have at least two prerequisite
// fields. Thus, SystemId, ComponentId, and SchemaVariantId have error permutations tests and SchemaId
// and PropId do not.

const SET_ID_VALUE: i64 = 1;
const UNSET_ID_VALUE: i64 = -1;

#[tokio::test]
async fn less_specific() {
    let context = AttributeResolverContextBuilder::new()
        .set_prop_id(1.into())
        .set_schema_id(2.into())
        .set_schema_variant_id(3.into())
        .set_component_id(4.into())
        .set_system_id(5.into())
        .to_context()
        .expect("cannot build resolver context");

    let new_context = context
        .less_specific()
        .expect("cannot create less specific context");

    assert_eq!(
        AttributeResolverContextBuilder::new()
            .set_prop_id(1.into())
            .set_schema_id(2.into())
            .set_schema_variant_id(3.into())
            .set_component_id(4.into())
            .to_context()
            .expect("cannot create expected context"),
        new_context,
    );

    let new_context = new_context
        .less_specific()
        .expect("cannot create less specific context");

    assert_eq!(
        AttributeResolverContextBuilder::new()
            .set_prop_id(1.into())
            .set_schema_id(2.into())
            .set_schema_variant_id(3.into())
            .to_context()
            .expect("cannot create expected context"),
        new_context,
    );

    let new_context = new_context
        .less_specific()
        .expect("cannot create less specific context");

    assert_eq!(
        AttributeResolverContextBuilder::new()
            .set_prop_id(1.into())
            .set_schema_id(2.into())
            .to_context()
            .expect("cannot create expected context"),
        new_context,
    );

    let new_context = new_context
        .less_specific()
        .expect("cannot create less specific context");

    assert_eq!(
        AttributeResolverContextBuilder::new()
            .set_prop_id(1.into())
            .to_context()
            .expect("cannot create expected context"),
        new_context,
    );

    let new_context = new_context
        .less_specific()
        .expect("cannot create less specific context");

    assert_eq!(
        AttributeResolverContextBuilder::new()
            .set_prop_id(1.into())
            .to_context()
            .expect("cannot create expected context"),
        new_context,
    );
}

#[tokio::test]
async fn builder_new() {
    let prop_id: PropId = SET_ID_VALUE.into();
    let schema_id: SchemaId = SET_ID_VALUE.into();
    let schema_variant_id: SchemaVariantId = SET_ID_VALUE.into();
    let component_id: ComponentId = SET_ID_VALUE.into();
    let system_id: SystemId = SET_ID_VALUE.into();

    let mut builder = AttributeResolverContextBuilder::new();
    let mut context = AttributeResolverContext::new();

    // Empty (PASS)
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );

    // SchemaId (FAIL) --> PropId (PASS)
    builder.set_schema_id(schema_id);
    assert!(builder.to_context().is_err());
    builder.unset_schema_id();
    context.set_prop_id(prop_id);
    builder.set_prop_id(prop_id);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );

    // SchemaVariantId (FAIL) --> SchemaId (PASS)
    builder.set_schema_variant_id(schema_variant_id);
    assert!(builder.to_context().is_err());
    builder.unset_schema_variant_id();
    context.set_schema_id(schema_id);
    builder.set_schema_id(schema_id);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );

    // ComponentId (FAIL) --> SchemaVariantId (PASS)
    builder.set_component_id(component_id);
    assert!(builder.to_context().is_err());
    builder.unset_component_id();
    context.set_schema_variant_id(schema_variant_id);
    builder.set_schema_variant_id(schema_variant_id);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );

    // SystemId (FAIL) --> ComponentId (PASS)
    builder.set_system_id(system_id);
    assert!(builder.to_context().is_err());
    builder.unset_system_id();
    context.set_component_id(component_id);
    builder.set_component_id(component_id);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );

    // SystemId (PASS)
    context.set_system_id(system_id);
    builder.set_system_id(system_id);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );
}

#[tokio::test]
async fn builder_from_context() {
    let prop_id: PropId = SET_ID_VALUE.into();
    let schema_id: SchemaId = SET_ID_VALUE.into();
    let schema_variant_id: SchemaVariantId = SET_ID_VALUE.into();
    let component_id: ComponentId = SET_ID_VALUE.into();
    let system_id: SystemId = SET_ID_VALUE.into();

    // Empty (PASS)
    let mut context = AttributeResolverContext::new();
    let builder = AttributeResolverContextBuilder::from_context(&context);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );

    // SchemaId (FAIL) --> PropId (PASS) [using previous builder]
    context = builder.to_context().expect("could not convert to context");
    context.set_schema_id(schema_id);
    let failure_builder = AttributeResolverContextBuilder::from_context(&context);
    assert!(failure_builder.to_context().is_err());
    context.set_schema_id(UNSET_ID_VALUE.into());
    context.set_prop_id(prop_id);
    let builder = AttributeResolverContextBuilder::from_context(&context);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );

    // SchemaVariantId (FAIL) --> SchemaId (PASS) [using previous builder]
    context = builder.to_context().expect("could not convert to context");
    context.set_schema_variant_id(schema_variant_id);
    let failure_builder = AttributeResolverContextBuilder::from_context(&context);
    assert!(failure_builder.to_context().is_err());
    context.set_schema_variant_id(UNSET_ID_VALUE.into());
    context.set_schema_id(schema_id);
    let builder = AttributeResolverContextBuilder::from_context(&context);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );

    // ComponentId (FAIL) --> SchemaVariantId (PASS) [using previous builder]
    context = builder.to_context().expect("could not convert to context");
    context.set_component_id(component_id);
    let failure_builder = AttributeResolverContextBuilder::from_context(&context);
    assert!(failure_builder.to_context().is_err());
    context.set_component_id(UNSET_ID_VALUE.into());
    context.set_schema_variant_id(schema_variant_id);
    let builder = AttributeResolverContextBuilder::from_context(&context);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );

    // SystemId (FAIL) --> ComponentId (PASS) [using previous builder]
    context = builder.to_context().expect("could not convert to context");
    context.set_system_id(system_id);
    let failure_builder = AttributeResolverContextBuilder::from_context(&context);
    assert!(failure_builder.to_context().is_err());
    context.set_system_id(UNSET_ID_VALUE.into());
    context.set_component_id(component_id);
    let builder = AttributeResolverContextBuilder::from_context(&context);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );

    // SystemId (PASS) [using previous builder]
    context = builder.to_context().expect("could not convert to context");
    context.set_system_id(system_id);
    let builder = AttributeResolverContextBuilder::from_context(&context);
    assert_eq!(
        builder.to_context().expect("could not convert to context"),
        context
    );
}

#[tokio::test]
async fn builder_system_id_error_permutations() {
    let prop_id: PropId = SET_ID_VALUE.into();
    let schema_id: SchemaId = SET_ID_VALUE.into();
    let schema_variant_id: SchemaVariantId = SET_ID_VALUE.into();
    let component_id: ComponentId = SET_ID_VALUE.into();
    let system_id: SystemId = SET_ID_VALUE.into();

    // ----------------
    // Prerequisites: 0
    // ----------------

    // ComponentId [ ] --> SchemaVariantId [ ] --> SchemaId [ ] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    assert!(builder.to_context().is_err());

    // ----------------
    // Prerequisites: 1
    // ----------------

    // ComponentId [x] --> SchemaVariantId [ ] --> SchemaId [ ] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_component_id(component_id);
    assert!(builder.to_context().is_err());

    // ComponentId [ ] --> SchemaVariantId [x] --> SchemaId [ ] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_schema_variant_id(schema_variant_id);
    assert!(builder.to_context().is_err());

    // ComponentId [ ] --> SchemaVariantId [ ] --> SchemaId [x] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_schema_id(schema_id);
    assert!(builder.to_context().is_err());

    // ComponentId [ ] --> SchemaVariantId [ ] --> SchemaId [ ] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());

    // ----------------
    // Prerequisites: 2
    // ----------------

    // ComponentId [x] --> SchemaVariantId [x] --> SchemaId [ ] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_component_id(component_id);
    builder.set_schema_variant_id(schema_variant_id);
    assert!(builder.to_context().is_err());

    // ComponentId [x] --> SchemaVariantId [ ] --> SchemaId [x] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_component_id(component_id);
    builder.set_schema_id(schema_id);
    assert!(builder.to_context().is_err());

    // ComponentId [x] --> SchemaVariantId [ ] --> SchemaId [ ] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_component_id(component_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());

    // ComponentId [ ] --> SchemaVariantId [x] --> SchemaId [x] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_schema_variant_id(schema_variant_id);
    builder.set_schema_id(schema_id);
    assert!(builder.to_context().is_err());

    // ComponentId [ ] --> SchemaVariantId [x] --> SchemaId [ ] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_schema_variant_id(schema_variant_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());

    // ComponentId [ ] --> SchemaVariantId [ ] --> SchemaId [x] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_schema_id(schema_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());

    // ----------------
    // Prerequisites: 3
    // ----------------

    // ComponentId [x] --> SchemaVariantId [x] --> SchemaId [x] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_component_id(component_id);
    builder.set_schema_variant_id(schema_variant_id);
    builder.set_schema_id(schema_id);
    assert!(builder.to_context().is_err());

    // ComponentId [x] --> SchemaVariantId [ ] --> SchemaId [x] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_component_id(component_id);
    builder.set_schema_id(schema_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());

    // ComponentId [x] --> SchemaVariantId [x] --> SchemaId [ ] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_component_id(component_id);
    builder.set_schema_variant_id(schema_variant_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());

    // ComponentId [ ] --> SchemaVariantId [x] --> SchemaId [x] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_system_id(system_id);
    builder.set_schema_variant_id(schema_variant_id);
    builder.set_schema_id(schema_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());
}

#[tokio::test]
async fn builder_component_id_error_permutations() {
    let prop_id: PropId = SET_ID_VALUE.into();
    let schema_id: SchemaId = SET_ID_VALUE.into();
    let schema_variant_id: SchemaVariantId = SET_ID_VALUE.into();
    let component_id: ComponentId = SET_ID_VALUE.into();

    // ----------------
    // Prerequisites: 0
    // ----------------

    // SchemaVariantId [ ] --> SchemaId [ ] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_component_id(component_id);
    assert!(builder.to_context().is_err());

    // ----------------
    // Prerequisites: 1
    // ----------------

    // SchemaVariantId [x] --> SchemaId [ ] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_component_id(component_id);
    builder.set_schema_variant_id(schema_variant_id);
    assert!(builder.to_context().is_err());

    // SchemaVariantId [ ] --> SchemaId [x] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_component_id(component_id);
    builder.set_schema_id(schema_id);
    assert!(builder.to_context().is_err());

    // SchemaVariantId [ ] --> SchemaId [ ] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_component_id(component_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());

    // ----------------
    // Prerequisites: 2
    // ----------------

    // SchemaVariantId [x] --> SchemaId [x] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_component_id(component_id);
    builder.set_schema_variant_id(schema_variant_id);
    builder.set_schema_id(schema_id);
    assert!(builder.to_context().is_err());

    // SchemaVariantId [x] --> SchemaId [ ] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_component_id(component_id);
    builder.set_schema_variant_id(schema_variant_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());

    // SchemaVariantId [ ] --> SchemaId [x] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_component_id(component_id);
    builder.set_schema_id(schema_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());
}

#[tokio::test]
async fn builder_schema_variant_id_error_permutations() {
    let prop_id: PropId = SET_ID_VALUE.into();
    let schema_id: SchemaId = SET_ID_VALUE.into();
    let schema_variant_id: SchemaVariantId = SET_ID_VALUE.into();

    // ----------------
    // Prerequisites: 0
    // ----------------

    // SchemaId [ ] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_schema_variant_id(schema_variant_id);
    assert!(builder.to_context().is_err());

    // ----------------
    // Prerequisites: 1
    // ----------------

    // SchemaId [x] --> PropId [ ]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_schema_variant_id(schema_variant_id);
    builder.set_schema_id(schema_id);
    assert!(builder.to_context().is_err());

    // SchemaId [ ] --> PropId [x]
    let mut builder = AttributeResolverContextBuilder::new();
    builder.set_schema_variant_id(schema_variant_id);
    builder.set_prop_id(prop_id);
    assert!(builder.to_context().is_err());
}
