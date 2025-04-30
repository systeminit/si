//! Provides macro(s) for building materialized views.

use manyhow::manyhow;
use quote::quote;

struct BuildMv {
    ctx: syn::Expr,
    frigg: syn::Expr,
    change: syn::Expr,
    mv_id: syn::Expr,
    mv_name: syn::Path,
    build_fn: syn::Expr,
}

impl syn::parse::Parse for BuildMv {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ctx = input.parse::<syn::Expr>()?;
        input.parse::<syn::Token![,]>()?;
        let frigg = input.parse::<syn::Expr>()?;
        input.parse::<syn::Token![,]>()?;
        let change = input.parse::<syn::Expr>()?;
        input.parse::<syn::Token![,]>()?;
        let mv_id = input.parse::<syn::Expr>()?;
        input.parse::<syn::Token![,]>()?;
        let mv_name = input.parse::<syn::Path>()?;
        input.parse::<syn::Token![,]>()?;
        let build_fn = input.parse::<syn::Expr>()?;
        input.parse::<Option<syn::Token![,]>>()?;

        Ok(BuildMv {
            ctx,
            frigg,
            change,
            mv_id,
            mv_name,
            build_fn,
        })
    }
}

#[manyhow]
#[proc_macro]
// You can also merge the two attributes: #[manyhow(proc_macro)]
pub fn build_mv(input: proc_macro2::TokenStream) -> syn::Result<proc_macro2::TokenStream> {
    let BuildMv {
        ctx,
        frigg,
        change,
        mv_id,
        mv_name,
        build_fn,
    } = syn::parse::<BuildMv>(input.into())?;

    let output = quote! {
        {
            let ctx = #ctx;
            let frigg = #frigg;
            let change = #change;
            let mv_id = #mv_id;

            let maybe_patch = if <#mv_name as si_frontend_types::materialized_view::MaterializedView>::trigger_entity() != change.entity_kind {
                None
            } else {
                if !ctx
                    .workspace_snapshot()?
                    .node_exists(change.entity_id)
                    .await
                {
                    // Object was removed
                    Some((
                        si_frontend_types::object::patch::ObjectPatch {
                            kind: <#mv_name as si_frontend_types::materialized_view::MaterializedView>::kind().to_string(),
                            id: mv_id,
                            // TODO: we need to get the prior version of this
                            from_checksum: Checksum::default().to_string(),
                            to_checksum: "0".to_string(),
                            patch: json_patch::Patch(vec![json_patch::PatchOperation::Remove(
                                json_patch::RemoveOperation::default(),
                            )]),
                        },
                        None
                    ))
                } else {
                    let mv = #build_fn?;
                    let mv_json = serde_json::to_value(&mv)?;
                    let to_checksum = si_frontend_types::checksum::FrontendChecksum::checksum(&mv).to_string();
                    let frontend_object: si_frontend_types::object::FrontendObject = mv.try_into()?;

                    let kind = <#mv_name as si_frontend_types::materialized_view::MaterializedView>::kind().to_string();
                    let (from_checksum, previous_data) = if let Some(previous_version) = frigg.get_current_object(ctx.workspace_pk()?, ctx.change_set_id(), &kind, &mv_id).await? {
                        (previous_version.checksum, previous_version.data)
                    } else {
                        // Object is new
                        ("0".to_string(), serde_json::Value::Null)
                    };

                    if from_checksum == to_checksum {
                        None
                    } else {
                        Some((
                            si_frontend_types::object::patch::ObjectPatch {
                                kind,
                                id: mv_id,
                                from_checksum,
                                to_checksum,
                                patch: json_patch::diff(&previous_data, &mv_json),
                            },
                            Some(frontend_object),
                        ))
                    }
                }
            };
            Ok(maybe_patch)
        }
    };

    Ok(output)
}
