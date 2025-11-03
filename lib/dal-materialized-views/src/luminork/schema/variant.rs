use std::collections::HashMap;

use dal::{
    DalContext,
    Func,
    FuncId,
    Prop,
    Schema,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    action::prototype::ActionPrototype,
    func::FuncKind,
    management::prototype::{
        ManagementFuncKind as dalMgmtFuncKind,
        ManagementPrototype,
    },
    schema::variant::root_prop::RootPropChild,
    workspace_snapshot::traits::prop::PropExt,
};
use si_frontend_mv_types::{
    luminork_schema_variant::LuminorkSchemaVariant as LuminorkSchemaVariantMv,
    luminork_schema_variant_func::{
        FuncKindVariant,
        LuminorkSchemaVariantFunc,
    },
    management::ManagementFuncKind,
};
use telemetry::prelude::*;

pub mod default;

#[instrument(
    name = "dal_materialized_views.luminork.schema.variant",
    level = "debug",
    skip_all
)]
pub async fn assemble(
    ctx: DalContext,
    id: SchemaVariantId,
) -> crate::Result<LuminorkSchemaVariantMv> {
    let schema_variant = SchemaVariant::get_by_id(&ctx, id).await?;

    let mut variant_func_ids = SchemaVariant::all_func_ids(&ctx, id).await?;
    let schema_id = SchemaVariant::schema_id(&ctx, id).await?;
    let overlay_func_ids = Schema::all_overlay_func_ids(&ctx, schema_id).await?;

    let func_details = build_func_details(
        &ctx,
        schema_id,
        schema_variant.id(),
        &variant_func_ids,
        &overlay_func_ids,
    )
    .await?;

    variant_func_ids.extend(overlay_func_ids);

    let domain_props = {
        let domain = Prop::find_prop_by_path(&ctx, id, &RootPropChild::Domain.prop_path()).await?;

        let prop_schema_tree = ctx
            .workspace_snapshot()?
            .build_prop_schema_tree(&ctx, domain.id)
            .await?;

        Some(prop_schema_tree)
    };

    let is_default_variant = SchemaVariant::is_default_by_id(&ctx, id).await?;
    let asset_func_id = schema_variant.asset_func_id_or_error()?;

    Ok(LuminorkSchemaVariantMv::new(
        id,
        schema_variant.display_name().into(),
        schema_variant.category().into(),
        schema_variant.color().into(),
        schema_variant.is_locked(),
        schema_variant.description(),
        schema_variant.link(),
        asset_func_id,
        variant_func_ids,
        func_details,
        is_default_variant,
        domain_props,
    ))
}

pub async fn build_func_details(
    ctx: &DalContext,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    variant_func_ids: &[FuncId],
    overlay_func_ids: &[FuncId],
) -> crate::Result<Vec<LuminorkSchemaVariantFunc>> {
    let mut variant_action_prototypes =
        ActionPrototype::for_variant(ctx, schema_variant_id).await?;
    let schema_action_prototypes = ActionPrototype::for_schema(ctx, schema_id).await?;
    variant_action_prototypes.extend(schema_action_prototypes);

    let management_prototypes =
        ManagementPrototype::list_for_schema_and_variant_id(ctx, schema_variant_id).await?;

    let mut index: HashMap<FuncId, FuncKindVariant> = HashMap::new();

    for action_prototype in variant_action_prototypes {
        let func_id = ActionPrototype::func_id(ctx, action_prototype.id).await?;
        index.insert(
            func_id,
            FuncKindVariant::Action(action_prototype.kind.into()),
        );
    }

    for mp in management_prototypes {
        let func_id = ManagementPrototype::func_id(ctx, mp.id).await?;
        let kind = ManagementPrototype::kind_by_id(ctx, mp.id).await?;
        index.insert(func_id, FuncKindVariant::Management(convert_kind(kind)));
    }

    let mut func_details = Vec::with_capacity(variant_func_ids.len() + overlay_func_ids.len());

    let mut func_ids_and_overlay_flag =
        Vec::with_capacity(variant_func_ids.len() + overlay_func_ids.len());
    func_ids_and_overlay_flag.extend(
        variant_func_ids
            .iter()
            .copied()
            .map(|id| (id, false /* not overlay */)),
    );
    func_ids_and_overlay_flag.extend(
        overlay_func_ids
            .iter()
            .copied()
            .map(|id| (id, true /* overlay */)),
    );

    for (func_id, is_overlay) in func_ids_and_overlay_flag {
        if let Some(kind) = index.remove(&func_id) {
            // We already know it's Action/Management
            func_details.push(LuminorkSchemaVariantFunc {
                id: func_id,
                func_kind: kind,
                is_overlay,
            });
        } else {
            // We are defaulting to other now
            let func = Func::get_by_id(ctx, func_id).await?;
            if func.kind != FuncKind::Intrinsic {
                func_details.push(LuminorkSchemaVariantFunc {
                    id: func.id,
                    func_kind: FuncKindVariant::Other(func.kind.into()),
                    is_overlay,
                });
            }
        }
    }

    Ok(func_details)
}

fn convert_kind(kind: dalMgmtFuncKind) -> ManagementFuncKind {
    match kind {
        dalMgmtFuncKind::Discover => ManagementFuncKind::Discover,
        dalMgmtFuncKind::Import => ManagementFuncKind::Import,
        dalMgmtFuncKind::Other => ManagementFuncKind::Other,
        dalMgmtFuncKind::RunTemplate => ManagementFuncKind::RunTemplate,
    }
}
