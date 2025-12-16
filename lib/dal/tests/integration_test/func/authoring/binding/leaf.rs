use dal::{
    AttributePrototype,
    DalContext,
    Prop,
    SchemaVariant,
    func::{
        authoring::FuncAuthoringClient,
        binding::{
            EventualParent,
            FuncBinding,
            leaf::LeafBinding,
        },
        intrinsics::IntrinsicFunc,
        leaf::{
            LeafInputLocation,
            LeafKind,
        },
    },
    schema::variant::authoring::VariantAuthoringClient,
};
use dal_test::{
    Result,
    prelude::{
        ChangeSetTestHelpers,
        OptionExt,
    },
    test,
};
use pretty_assertions_sorted::assert_eq;
use si_frontend_types::LeafBindingPrototype;

#[test]
async fn delete_then_create_binding(ctx: &mut DalContext) -> Result<()> {
    let schema_variant_id = SchemaVariant::default_id_for_schema_name(ctx, "swifty").await?;

    // Create an unlocked copy before modifying the bindings and commit.
    let schema_variant_id =
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, schema_variant_id)
            .await?
            .id();
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Add a new leaf func and commit.
    let name = "the Divinity trailer was nasty".to_string();
    let func = FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(name.clone()),
        LeafKind::CodeGeneration,
        EventualParent::SchemaVariant(schema_variant_id),
        &[
            LeafInputLocation::Domain,
            LeafInputLocation::Code,
            LeafInputLocation::Resource,
            LeafInputLocation::Secrets,
        ],
    )
    .await?;
    FuncAuthoringClient::update_func(ctx, func.id, Some(name.clone()), None).await?;
    FuncAuthoringClient::save_code(
        ctx,
        func.id,
        "async function main(input) { return { format: 'json', code: '{}' }; }",
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Delete the binding and commit.
    let bindings =
        FuncBinding::get_bindings_for_schema_variant_id(ctx, func.id, schema_variant_id).await?;
    assert_eq!(
        1,              // expected
        bindings.len()  // actual
    );
    let attribute_prototype_id = bindings
        .iter()
        .find_map(|binding| {
            if let FuncBinding::CodeGeneration(leaf_binding) = binding {
                assert_eq!(
                    LeafKind::CodeGeneration, // expected
                    leaf_binding.leaf_kind    // actual
                );
                if let LeafBindingPrototype::Attribute(attribute_prototype_id) =
                    leaf_binding.leaf_binding_prototype
                {
                    return Some(attribute_prototype_id);
                }
            }
            None
        })
        .ok_or_eyre("attribute prototype not found from bindings")?;
    let prop_id = AttributePrototype::prop_id(ctx, attribute_prototype_id)
        .await?
        .ok_or_eyre("could not find prop for prototype")?;

    // In case the variant has more bindings, delete them too.
    let remaining_prototypes = Prop::prototypes_by_key(ctx, prop_id).await?;
    for (_, id) in remaining_prototypes {
        LeafBinding::delete_leaf_func_binding(ctx, id).await?;
    }

    // Ensure that we only have one prototype and it is the default one that was created after all were deleted.
    let mut remaining_prototypes = Prop::prototypes_by_key(ctx, prop_id).await?;
    let remaining_prototype = remaining_prototypes
        .pop()
        .expect("no remaining prototypes left");
    assert!(remaining_prototypes.is_empty());
    let func = AttributePrototype::func(ctx, remaining_prototype.1).await?;
    assert!(func.is_intrinsic());
    assert_eq!(
        IntrinsicFunc::Unset.name(), // expected
        &func.name                   // actual
    );

    // We can now commit our changes.
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Ensure that building the schema variant MV works.
    dal_materialized_views::schema_variant::assemble(ctx.clone(), schema_variant_id)
        .await
        .expect("could not build schema variant MV");

    // Add another leaf func and commit.
    let name = "but hey, it looked good!".to_string();
    let func = FuncAuthoringClient::create_new_leaf_func(
        ctx,
        Some(name.clone()),
        LeafKind::CodeGeneration,
        EventualParent::SchemaVariant(schema_variant_id),
        &[
            LeafInputLocation::Domain,
            LeafInputLocation::Code,
            LeafInputLocation::Resource,
            LeafInputLocation::Secrets,
        ],
    )
    .await?;
    FuncAuthoringClient::update_func(ctx, func.id, Some(name.clone()), None).await?;
    FuncAuthoringClient::save_code(
        ctx,
        func.id,
        "async function main(input) { return { format: 'json', code: '{}' }; }",
    )
    .await?;
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;

    // Ensure that we have one binding (the default was replaced) and that the schema variant MV
    // works, again.
    let mut bindings =
        FuncBinding::get_bindings_for_schema_variant_id(ctx, func.id, schema_variant_id).await?;
    let binding = bindings.pop().expect("no bindings found");
    assert!(bindings.is_empty());
    assert!(!binding.is_overlay());
    dal_materialized_views::schema_variant::assemble(ctx.clone(), schema_variant_id)
        .await
        .expect("could not build schema variant MV");

    Ok(())
}
