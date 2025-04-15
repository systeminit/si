use std::collections::HashSet;

use darling::FromAttributes;
use manyhow::{bail, emit, manyhow};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Path};

#[manyhow]
#[proc_macro_derive(FrontendChecksum, attributes(frontend_checksum))]
pub fn frontend_checksum_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_frontend_checksum(input, errors)
}

fn derive_frontend_checksum(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident,
        data: type_data,
        ..
    } = input.clone();

    match &type_data {
        Data::Struct(struct_data) => derive_frontend_checksum_struct(ident, struct_data, errors),
        Data::Enum(_) => derive_frontend_checksum_enum(ident),
        _ => bail!("FrontendChecksum can only be derived for structs and enums"),
    }
}

fn derive_frontend_checksum_struct(
    ident: syn::Ident,
    struct_data: &syn::DataStruct,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let mut field_update_parts = Vec::new();
    for field in &struct_data.fields {
        let Some(field_ident) = &field.ident else {
            emit!(
                errors,
                syn::Error::new_spanned(field, "struct field must have an identifier")
            );
            continue;
        };
        field_update_parts.push(
            quote! { hasher.update(FrontendChecksum::checksum(&self.#field_ident).as_bytes()); },
        )
    }
    errors.into_result()?;

    let mut field_updates = TokenStream::new();
    field_updates.extend(field_update_parts);

    let checksum_fn = quote! {
        fn checksum(&self) -> Checksum {
            let mut hasher = ChecksumHasher::new();
            #field_updates
            hasher.finalize()
        }
    };

    let output = quote! {
        impl FrontendChecksum for #ident {
            #checksum_fn
        }
    };

    Ok(output.into())
}

fn derive_frontend_checksum_enum(ident: syn::Ident) -> manyhow::Result<proc_macro::TokenStream> {
    let checksum_fn = quote! {
        fn checksum(&self) -> Checksum {
            let mut hasher = ChecksumHasher::new();
            hasher.update(self.to_string().as_bytes());
            hasher.finalize()
        }
    };

    let output = quote! {
        impl FrontendChecksum for #ident {
            #checksum_fn
        }
    };

    Ok(output.into())
}

#[manyhow]
#[proc_macro_derive(FrontendObject, attributes(frontend_object))]
pub fn frontend_object_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_frontend_object(input, errors)
}

fn derive_frontend_object(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident,
        data: type_data,
        ..
    } = input.clone();

    if !matches!(type_data, Data::Struct(_)) {
        emit!(
            errors,
            input,
            "FrontendObject can only be derived for structs"
        );
    }
    errors.into_result()?;

    let output = quote! {
        impl ::std::convert::TryFrom<#ident> for FrontendObject {
            type Error = ::serde_json::Error;

            fn try_from(value: #ident) -> ::std::result::Result<Self, Self::Error> {
                let kind = ReferenceKind::#ident.to_string();
                let id = value.id.to_string();
                let checksum = FrontendChecksum::checksum(&value).to_string();
                let data = ::serde_json::to_value(value)?;

                Ok(FrontendObject {
                    kind,
                    id,
                    checksum,
                    data,
                })
            }
        }
    };

    Ok(output.into())
}

#[manyhow]
#[proc_macro_derive(Refer, attributes(refer))]
pub fn refer_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_refer(input, errors)
}

fn derive_refer(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident,
        data: type_data,
        ..
    } = input.clone();

    let Data::Struct(struct_data) = type_data else {
        bail!("Refer can only be derived for structs");
    };

    let mut id_type = None;
    let mut id_field = None;
    for field in &struct_data.fields {
        let Some(field_ident) = &field.ident else {
            continue;
        };
        let field_ty = &field.ty;

        if field_ident == "id" {
            id_type = Some(field_ty.clone());
            id_field = Some(field_ident.clone());
        }
    }
    errors.into_result()?;

    let Some(id_field) = id_field else {
        bail!(input, "'id' field must be present");
    };
    let Some(id_type) = id_type else {
        bail!(input, "'id' field must have a type");
    };

    let refer_impl = quote! {
        impl Refer<#id_type> for #ident {
            fn reference_kind(&self) -> ReferenceKind {
                self.into()
            }

            fn reference_id(&self) -> ReferenceId<#id_type> {
                ReferenceId(self.#id_field)
            }
        }
    };

    let from_for_reference_impl = quote! {
        impl From<&#ident> for Reference<#id_type> {
            fn from(value: &#ident) -> Self {
                value.reference()
            }
        }
    };

    let from_for_reference_kind_impl = quote! {
        impl From<&#ident> for ReferenceKind {
            fn from(value: &#ident) -> Self {
                ReferenceKind::#ident
            }
        }
    };

    let output = quote! {
        #refer_impl
        #from_for_reference_impl
        #from_for_reference_kind_impl
    };

    Ok(output.into())
}

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(mv))]
struct MaterializedViewOptions {
    trigger_entity: Option<Path>,
    reference_kind: Option<Path>,
}

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(mv))]
struct MaterializedViewReferenceOptions {
    reference_kind: Option<Path>,
}

#[manyhow]
#[proc_macro_derive(MV, attributes(mv))]
pub fn materialized_view_derive(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    derive_materialized_view(input, errors)
}

fn derive_materialized_view(
    input: proc_macro::TokenStream,
    _errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    let input = syn::parse::<DeriveInput>(input)?;
    let DeriveInput {
        ident,
        data: type_data,
        attrs,
        ..
    } = input.clone();
    let struct_options = MaterializedViewOptions::from_attributes(&attrs)?;

    let Data::Struct(struct_data) = &type_data else {
        bail!("MV can only be derived for structs");
    };

    let mut reference_kinds: HashSet<Path> = HashSet::new();
    for field in &struct_data.fields {
        let field_attrs = MaterializedViewReferenceOptions::from_attributes(&field.attrs)?;
        if let Some(reference_kind) = &field_attrs.reference_kind {
            reference_kinds.insert(reference_kind.clone());
        }
    }

    let Some(trigger_entity) = struct_options.trigger_entity else {
        bail!(input, "MV must have a trigger_entity attribute");
    };
    let Some(self_reference_kind) = struct_options.reference_kind else {
        bail!(input, "MV must have a reference_kind attribute");
    };

    let mut sorted_reference_kinds: Vec<Path> = reference_kinds.into_iter().collect();

    sorted_reference_kinds.sort_by_cached_key(path_to_string);

    let output = quote! {
        impl MaterializedView for #ident {
            fn kind() -> ReferenceKind {
                #self_reference_kind
            }

            fn reference_dependencies() -> &'static [ReferenceKind] {
                &[#(#sorted_reference_kinds),*]
            }

            fn trigger_entity() -> EntityKind {
                #trigger_entity
            }
        }
    };

    Ok(output.into())
}

fn path_to_string(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

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

    let build_mv_ident = format_ident!(
        "build_mv_{}",
        mv_name
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>()
            .join("_")
    );

    let output = quote! {
        {
            async fn #build_mv_ident(
                ctx: &DalContext,
                frigg: &frigg::FriggStore,
                change: &Change,
                mv_id: String,
            ) -> Result<(
                Option<si_frontend_types::object::patch::ObjectPatch>,
                Option<si_frontend_types::object::FrontendObject>,
            ), MaterializedViewError> {
                if <#mv_name as si_frontend_types::materialized_view::MaterializedView>::trigger_entity() == change.entity_kind {
                    if !ctx
                        .workspace_snapshot()?
                        .node_exists(change.entity_id)
                        .await
                    {
                        // Object was removed
                        return Ok((
                            Some(si_frontend_types::object::patch::ObjectPatch {
                                kind: <#mv_name as si_frontend_types::materialized_view::MaterializedView>::kind().to_string(),
                                id: mv_id,
                                // TODO: we need to get the prior version of this
                                from_checksum: Checksum::default().to_string(),
                                to_checksum: "0".to_string(),
                                patch: json_patch::Patch(vec![json_patch::PatchOperation::Remove(
                                    json_patch::RemoveOperation::default(),
                                )]),
                            }),
                            None,
                        ));
                    }

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

                    Ok((
                        Some(si_frontend_types::object::patch::ObjectPatch {
                            kind,
                            id: mv_id,
                            from_checksum,
                            to_checksum,
                            patch: json_patch::diff(&previous_data, &mv_json),
                        }),
                        Some(frontend_object),
                    ))
                } else {
                    Ok((None, None))
                }
            }

            #build_mv_ident(#ctx, #frigg, #change, #mv_id)
        }
    };

    Ok(output)
}
