use dal::{
    prop::PropPath, prop_tree::PropTree, DalContext, Prop, Schema, SchemaVariant, SchemaVariantId,
    StandardModel,
};
use dal_test::test;
use pretty_assertions_sorted::assert_eq;

#[test]
async fn all_schema_variants(ctx: &DalContext) {
    let prop_tree = PropTree::new(ctx, true, None, None)
        .await
        .expect("able to fetch prop tree");

    let variant_ids: Vec<SchemaVariantId> = prop_tree
        .root_props
        .iter()
        .map(|node| node.schema_variant_id)
        .collect();

    let all_variants = SchemaVariant::list(ctx)
        .await
        .expect("able to list schema variants");

    assert_eq!(all_variants.len(), variant_ids.len());
}

#[test]
async fn one_schema_variant(ctx: &DalContext) {
    let starfield = Schema::find_by_attr(ctx, "name", &"starfield")
        .await
        .expect("get starfield")
        .pop()
        .expect("starfield is there");

    let default_variant = starfield
        .default_variant(ctx)
        .await
        .expect("get default variant of starfield");

    let prop_tree = PropTree::new(ctx, false, Some(vec![*default_variant.id()]), None)
        .await
        .expect("able to fetch prop tree");

    assert_eq!(1, prop_tree.root_props.len());
    let root_prop = prop_tree.root_props.get(0).expect("able to get root prop");
    assert_eq!(*default_variant.id(), root_prop.schema_variant_id);

    let domain = root_prop
        .children
        .iter()
        .find(|node| node.name.as_str() == "domain")
        .expect("able to find domain");

    assert!(
        !domain
            .children
            .iter()
            .any(|node| node.name.as_str() == "hidden_prop"),
        "hidden props remain hidden"
    );

    let prop_tree = PropTree::new(ctx, true, Some(vec![*default_variant.id()]), None)
        .await
        .expect("able to fetch prop tree");
    let root_prop = prop_tree.root_props.get(0).expect("able to get root prop");
    let domain = root_prop
        .children
        .iter()
        .find(|node| node.name.as_str() == "domain")
        .expect("able to find domain");

    assert!(
        domain
            .children
            .iter()
            .any(|node| node.name.as_str() == "hidden_prop"),
        "hidden props revealed when asked"
    );
}

#[test]
pub async fn typescript_generation(ctx: &DalContext) {
    let starfield = Schema::find_by_attr(ctx, "name", &"starfield")
        .await
        .expect("get starfield")
        .pop()
        .expect("starfield is there");

    let default_variant = starfield
        .default_variant(ctx)
        .await
        .expect("get default variant of starfield");

    let prop_tree = PropTree::new(ctx, true, Some(vec![*default_variant.id()]), None)
        .await
        .expect("able to fetch prop tree");

    let mut typescript_iface = prop_tree
        .ts_types(ctx)
        .await
        .expect("able to generate TS interface");

    assert_eq!(1, typescript_iface.len());

    let (type_name, _ts_type) = typescript_iface.pop().expect("has an interface");

    assert_eq!("Starfield_V0_Root", &type_name);

    let si_prop =
        Prop::find_prop_by_path(ctx, *default_variant.id(), &PropPath::new(["root", "si"]))
            .await
            .expect("able to find prop");

    let prop_tree = PropTree::new(
        ctx,
        true,
        Some(vec![*default_variant.id()]),
        Some(*si_prop.id()),
    )
    .await
    .expect("able to fetch prop tree");

    let mut typescript_iface = prop_tree
        .ts_types(ctx)
        .await
        .expect("able to generate TS interface for root/si");

    let (type_name, ts_type) = typescript_iface.pop().expect("has an interface");

    assert_eq!("Starfield_V0_Si", &type_name);

    assert_eq!(
        "interface Starfield_V0_Si {
\"color\": string | null | undefined;
\"name\": string | null | undefined;
\"protected\": boolean | null | undefined;
\"type\": string | null | undefined;
}",
        &ts_type
    );

    // test a single, non object prop type generation
    let si_prop = Prop::find_prop_by_path(
        ctx,
        *default_variant.id(),
        &PropPath::new(["root", "si", "protected"]),
    )
    .await
    .expect("able to find prop");

    let prop_tree = PropTree::new(
        ctx,
        true,
        Some(vec![*default_variant.id()]),
        Some(*si_prop.id()),
    )
    .await
    .expect("able to fetch prop tree");

    let mut typescript_iface = prop_tree
        .ts_types(ctx)
        .await
        .expect("able to generate TS interface for root/si");

    let (type_name, ts_type) = typescript_iface.pop().expect("has an interface");
    assert_eq!("Starfield_V0_Protected", &type_name);
    assert_eq!(
        "type Starfield_V0_Protected = boolean | null | undefined;",
        &ts_type
    );

    // test a single, non object prop type generation (maps)
    let si_prop = Prop::find_prop_by_path(
        ctx,
        *default_variant.id(),
        &PropPath::new(["root", "qualification"]),
    )
    .await
    .expect("able to find prop");

    let prop_tree = PropTree::new(
        ctx,
        true,
        Some(vec![*default_variant.id()]),
        Some(*si_prop.id()),
    )
    .await
    .expect("able to fetch prop tree");

    let mut typescript_iface = prop_tree
        .ts_types(ctx)
        .await
        .expect("able to generate TS interface for root/si");

    let (type_name, ts_type) = typescript_iface.pop().expect("has an interface");
    eprintln!("{}", &ts_type);

    assert_eq!("Starfield_V0_Qualification", &type_name);
    assert_eq!(
        "type Starfield_V0_Qualification = Record<string, {
\"message\": string | null | undefined;
\"result\": string | null | undefined;
}> | null | undefined;",
        &ts_type
    );
}
