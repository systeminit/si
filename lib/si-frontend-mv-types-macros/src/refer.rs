use manyhow::bail;
use quote::quote;
use syn::{
    Data,
    DeriveInput,
};

pub fn derive_refer(
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
        impl crate::reference::Refer<#id_type> for #ident {
            fn reference_kind(&self) -> crate::reference::ReferenceKind {
                self.into()
            }

            fn reference_id(&self) -> crate::reference::ReferenceId<#id_type> {
                crate::reference::ReferenceId(self.#id_field)
            }
        }
    };

    let from_for_reference_impl = quote! {
        impl From<&#ident> for crate::reference::Reference<#id_type> {
            fn from(value: &#ident) -> Self {
                crate::reference::Refer::reference(value)
            }
        }
    };

    let from_for_reference_kind_impl = quote! {
        impl From<&#ident> for crate::reference::ReferenceKind {
            fn from(value: &#ident) -> Self {
                crate::reference::ReferenceKind::#ident
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
