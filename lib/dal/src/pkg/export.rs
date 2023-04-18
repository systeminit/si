use std::{
    collections::{hash_map::Entry, HashMap},
    convert::TryFrom,
    path::PathBuf,
};
use strum::IntoEnumIterator;

use si_pkg::{
    FuncSpec, FuncSpecBackendKind, FuncSpecBackendResponseType, LeafFunctionSpec,
    LeafInputLocation as PkgLeafInputLocation, PkgSpec, PropSpec, PropSpecBuilder, PropSpecKind,
    SchemaSpec, SchemaVariantSpec, SchemaVariantSpecBuilder, SiPkg, SpecError,
};

use crate::{
    func::argument::FuncArgument,
    prop_tree::{PropTree, PropTreeNode},
    DalContext, Func, FuncId, LeafKind, PropId, PropKind, Schema, SchemaVariant, SchemaVariantId,
    StandardModel, StandardModelError,
};

use super::{PkgError, PkgResult};

// TODO(fnichol): another first-pass function with arguments. At the moment we're passing a list of
// `SchemaVariantId`s in an effort to export specific schema/variant combos but this will change in
// the future to be more encompassing. And yes, to many function args, way too many--and they're
// all `String`s
pub async fn export_pkg(
    ctx: &DalContext,
    pkg_file_path: impl Into<PathBuf>,
    name: impl Into<String>,
    version: impl Into<String>,
    description: Option<impl Into<String>>,
    created_by: impl Into<String>,
    variant_ids: Vec<SchemaVariantId>,
) -> PkgResult<()> {
    let mut pkg_spec_builder = PkgSpec::builder();
    pkg_spec_builder
        .name(name)
        .version(version)
        .created_by(created_by);
    if let Some(description) = description {
        pkg_spec_builder.description(description);
    }

    let mut func_specs = HashMap::new();

    for variant_id in variant_ids {
        let related_funcs = SchemaVariant::all_funcs(ctx, variant_id).await?;
        for func in &related_funcs {
            if !func_specs.contains_key(func.id()) {
                let func_spec = build_func_spec(func).await?;
                func_specs.insert(*func.id(), func_spec.clone());
                pkg_spec_builder.func(func_spec);
            }
        }
        let schema_spec = build_schema_spec(ctx, variant_id, &func_specs).await?;
        pkg_spec_builder.schema(schema_spec);
    }

    let spec = pkg_spec_builder.build()?;

    let pkg = SiPkg::load_from_spec(spec)?;
    pkg.write_to_file(pkg_file_path).await?;

    Ok(())
}

async fn build_func_spec(func: &Func) -> Result<FuncSpec, PkgError> {
    let mut func_spec_builder = FuncSpec::builder();

    func_spec_builder.name(func.name());

    if let Some(display_name) = func.display_name() {
        func_spec_builder.display_name(display_name);
    }

    if let Some(description) = func.description() {
        func_spec_builder.description(description);
    }

    if let Some(link) = func.link() {
        func_spec_builder.try_link(link)?;
    }
    // Should we package an empty func?
    func_spec_builder.handler(func.handler().unwrap_or(""));
    func_spec_builder.code_base64(func.code_base64().unwrap_or(""));

    func_spec_builder.response_type(FuncSpecBackendResponseType::try_from(
        *func.backend_response_type(),
    )?);

    func_spec_builder.backend_kind(FuncSpecBackendKind::try_from(*func.backend_kind())?);

    func_spec_builder.hidden(func.hidden());

    Ok(func_spec_builder.build()?)
}

async fn build_schema_spec(
    ctx: &DalContext,
    variant_id: SchemaVariantId,
    func_specs: &HashMap<FuncId, FuncSpec>,
) -> Result<SchemaSpec, PkgError> {
    let (variant, schema) = get_schema_and_variant(ctx, variant_id).await?;

    let mut schema_spec_builder = SchemaSpec::builder();
    schema_spec_builder.name(schema.name());
    set_schema_spec_category_data(ctx, &schema, &mut schema_spec_builder).await?;

    let variant_spec = build_variant_spec(ctx, variant, func_specs).await?;
    schema_spec_builder.variant(variant_spec);

    let schema_spec = schema_spec_builder.build()?;

    Ok(schema_spec)
}

async fn build_variant_spec(
    ctx: &DalContext,
    variant: SchemaVariant,
    func_specs: &HashMap<FuncId, FuncSpec>,
) -> Result<SchemaVariantSpec, PkgError> {
    let mut variant_spec_builder = SchemaVariantSpec::builder();
    variant_spec_builder.name(variant.name());
    if let Some(color_str) = variant.color(ctx).await? {
        variant_spec_builder.color(color_str);
    };
    if let Some(link) = variant.link() {
        variant_spec_builder.try_link(link)?;
    }
    set_variant_spec_prop_data(ctx, &variant, &mut variant_spec_builder).await?;

    for leaf_kind in LeafKind::iter() {
        for leaf_func in
            SchemaVariant::find_leaf_item_functions(ctx, *variant.id(), leaf_kind).await?
        {
            let func_spec = func_specs
                .get(leaf_func.id())
                .ok_or(PkgError::MissingExportedFunc(*leaf_func.id()))?;

            let mut inputs = vec![];
            for arg in FuncArgument::list_for_func(ctx, *leaf_func.id()).await? {
                inputs.push(PkgLeafInputLocation::try_from_arg_name(arg.name())?);
            }

            let leaf_func_spec = LeafFunctionSpec::builder()
                .func_unique_id(func_spec.unique_id)
                .leaf_kind(leaf_kind)
                .inputs(inputs)
                .build()?;

            variant_spec_builder.leaf_function(leaf_func_spec);
        }
    }

    let variant_spec = variant_spec_builder.build()?;

    Ok(variant_spec)
}

async fn get_schema_and_variant(
    ctx: &DalContext,
    variant_id: SchemaVariantId,
) -> Result<(SchemaVariant, Schema), PkgError> {
    let variant = SchemaVariant::get_by_id(ctx, &variant_id)
        .await?
        .ok_or_else(|| {
            StandardModelError::ModelMissing("schema_variants".to_string(), variant_id.to_string())
        })?;

    let schema = variant.schema(ctx).await?.ok_or_else(|| {
        PkgError::StandardModelMissingBelongsTo(
            "schema_variant_belongs_to_schema",
            "schema_variant",
            variant_id.to_string(),
        )
    })?;

    Ok((variant, schema))
}

async fn set_schema_spec_category_data(
    ctx: &DalContext,
    schema: &Schema,
    schema_spec_builder: &mut si_pkg::SchemaSpecBuilder,
) -> Result<(), PkgError> {
    let mut schema_ui_menus = schema.ui_menus(ctx).await?;
    let schema_ui_menu = schema_ui_menus.pop().ok_or_else(|| {
        PkgError::StandardModelMissingBelongsTo(
            "schema_ui_menu_belongs_to_schema",
            "schema",
            (*schema.id()).to_string(),
        )
    })?;
    if !schema_ui_menus.is_empty() {
        return Err(PkgError::StandardModelMultipleBelongsTo(
            "schema_ui_menu_belongs_to_schema",
            "schema",
            (*schema.id()).to_string(),
        ));
    }

    schema_spec_builder.category(schema_ui_menu.category());
    schema_spec_builder.category_name(schema_ui_menu.name());

    Ok(())
}

async fn set_variant_spec_prop_data(
    ctx: &DalContext,
    variant: &SchemaVariant,
    variant_spec: &mut SchemaVariantSpecBuilder,
) -> Result<(), PkgError> {
    let mut prop_tree = PropTree::new(ctx, false, Some(*variant.id())).await?;
    let root_tree_node = prop_tree
        .root_props
        .pop()
        .ok_or_else(|| PkgError::prop_tree_invalid("root prop not found"))?;
    if !prop_tree.root_props.is_empty() {
        return Err(PkgError::prop_tree_invalid(
            "prop tree contained multiple root props",
        ));
    }
    let domain_tree_node = root_tree_node
        .children
        .into_iter()
        .find(|tree_node| tree_node.name == "domain" && tree_node.path == "/root/")
        .ok_or_else(|| PkgError::prop_tree_invalid("domain prop not found"))?;

    #[derive(Debug)]
    struct TraversalStackEntry {
        builder: PropSpecBuilder,
        prop_id: PropId,
        parent_prop_id: Option<PropId>,
    }

    let mut stack: Vec<(PropTreeNode, Option<PropId>)> = Vec::new();
    for domain_child_tree_node in domain_tree_node.children {
        stack.push((domain_child_tree_node, None));
    }

    let mut traversal_stack: Vec<TraversalStackEntry> = Vec::new();

    while let Some((tree_node, parent_prop_id)) = stack.pop() {
        let prop_id = tree_node.prop_id;
        let mut builder = PropSpec::builder();
        builder
            .kind(match tree_node.kind {
                PropKind::Array => PropSpecKind::Array,
                PropKind::Boolean => PropSpecKind::Boolean,
                PropKind::Integer => PropSpecKind::Number,
                PropKind::Object => PropSpecKind::Object,
                PropKind::String => PropSpecKind::String,
                PropKind::Map => PropSpecKind::Map,
            })
            .name(tree_node.name);
        traversal_stack.push(TraversalStackEntry {
            builder,
            prop_id,
            parent_prop_id,
        });

        for child_tree_node in tree_node.children {
            stack.push((child_tree_node, Some(prop_id)));
        }
    }

    let mut prop_children_map: HashMap<PropId, Vec<PropSpec>> = HashMap::new();

    while let Some(mut entry) = traversal_stack.pop() {
        if let Some(mut prop_children) = prop_children_map.remove(&entry.prop_id) {
            match entry.builder.get_kind() {
                Some(kind) => match kind {
                    PropSpecKind::Object => {
                        entry.builder.entries(prop_children);
                    }
                    PropSpecKind::Map | PropSpecKind::Array => {
                        let type_prop = prop_children.pop().ok_or_else(|| {
                            PkgError::prop_spec_children_invalid(format!(
                                "found no child for map/array for prop id {}",
                                entry.prop_id,
                            ))
                        })?;
                        if !prop_children.is_empty() {
                            return Err(PkgError::prop_spec_children_invalid(format!(
                                "found multiple children for map/array for prop id {}",
                                entry.prop_id,
                            )));
                        }
                        entry.builder.type_prop(type_prop);
                    }
                    PropSpecKind::String | PropSpecKind::Number | PropSpecKind::Boolean => {
                        return Err(PkgError::prop_spec_children_invalid(format!(
                            "primitve prop type should have no children for prop id {}",
                            entry.prop_id,
                        )));
                    }
                },
                None => {
                    return Err(SpecError::UninitializedField("kind").into());
                }
            };
        }

        let prop_spec = entry.builder.build()?;

        match entry.parent_prop_id {
            None => {
                variant_spec.prop(prop_spec);
            }
            Some(parent_prop_id) => {
                match prop_children_map.entry(parent_prop_id) {
                    Entry::Occupied(mut occupied) => {
                        occupied.get_mut().push(prop_spec);
                    }
                    Entry::Vacant(vacant) => {
                        vacant.insert(vec![prop_spec]);
                    }
                };
            }
        };
    }

    Ok(())
}
