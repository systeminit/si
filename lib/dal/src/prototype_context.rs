use crate::{
    ComponentId, DalContext, FuncError, FuncId, SchemaId, SchemaVariantId, StandardModel,
    StandardModelError, TenancyError, TransactionsError,
};
use std::future::Future;
use thiserror::Error;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PrototypeContextError {
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error("func {0} not found")]
    FuncNotFound(FuncId),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
}

pub type PrototypeContextResult<T> = Result<T, PrototypeContextError>;

/// Represents a context for a prototype that has the component_id as the max specificity.
/// Useful for using the various PrototypeContext objects generically. Does not apply to
/// AttributePrototypeContexts!
pub trait PrototypeContext {
    fn component_id(&self) -> ComponentId;
    fn set_component_id(&mut self, component_id: ComponentId);

    fn schema_id(&self) -> SchemaId;
    fn set_schema_id(&mut self, schema_id: SchemaId);

    fn schema_variant_id(&self) -> SchemaVariantId;
    fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId);
}

/// A helper trait for objects that have a [`PrototypeContext`] associated with them.
pub trait HasPrototypeContext<T>
where
    T: PrototypeContext,
{
    fn context(&self) -> T;

    fn new_context() -> T;

    fn new_context_for_context_field(context_field: PrototypeContextField) -> T {
        let mut context = Self::new_context();
        match context_field {
            PrototypeContextField::Schema(schema_id) => {
                context.set_schema_id(schema_id);
            }
            PrototypeContextField::SchemaVariant(schema_variant_id) => {
                context.set_schema_variant_id(schema_variant_id);
            }
            PrototypeContextField::Component(component_id) => {
                context.set_component_id(component_id);
            }
        }

        context
    }
}

#[remain::sorted]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PrototypeContextField {
    Component(ComponentId),
    Schema(SchemaId),
    SchemaVariant(SchemaVariantId),
}

impl From<ComponentId> for PrototypeContextField {
    fn from(component_id: ComponentId) -> Self {
        PrototypeContextField::Component(component_id)
    }
}

impl From<SchemaId> for PrototypeContextField {
    fn from(schema_id: SchemaId) -> Self {
        PrototypeContextField::Schema(schema_id)
    }
}

impl From<SchemaVariantId> for PrototypeContextField {
    fn from(schema_variant_id: SchemaVariantId) -> Self {
        PrototypeContextField::SchemaVariant(schema_variant_id)
    }
}

/// Given a list of existing prototypes and new prototype context fields, ensure that only the
/// requested prototypes exist with the associations called for in `prototype_context_field_ids`.
/// If new prototypes need to be created, they have to be created via the callback passed to
/// `create_new_prototype_callback`.
pub async fn associate_prototypes<'a, 'b, T, P, C, F>(
    ctx: &'b DalContext,
    existing_protos_for_func: &mut [P],
    prototype_context_field_ids: &[T],
    create_new_prototype_callback: Box<
        dyn Fn(DalContext, PrototypeContextField) -> F + Send + Sync + 'a,
    >,
) -> PrototypeContextResult<()>
where
    F: Future<Output = PrototypeContextResult<()>>,
    T: Into<PrototypeContextField> + Copy,
    P: HasPrototypeContext<C> + StandardModel + Send + Sync + Sized,
    C: PrototypeContext,
{
    let mut existing_field_ids: Vec<PrototypeContextField> = vec![];
    let prototype_context_field_ids: Vec<PrototypeContextField> = prototype_context_field_ids
        .iter()
        .map(|field| (*field).into())
        .collect();

    for proto in existing_protos_for_func {
        let component_id = proto.context().component_id();
        let schema_variant_id = proto.context().schema_variant_id();

        if component_id.is_none() && schema_variant_id.is_none() {
            continue;
        }

        if component_id.is_some() && !prototype_context_field_ids.contains(&component_id.into())
            || schema_variant_id.is_some()
                && !prototype_context_field_ids.contains(&schema_variant_id.into())
        {
            proto.delete_by_id(ctx).await?;
            continue;
        } else if component_id.is_some() {
            existing_field_ids.push(component_id.into());
        } else if schema_variant_id.is_some() {
            existing_field_ids.push(schema_variant_id.into());
        }
    }

    for desired in &prototype_context_field_ids {
        if existing_field_ids.contains(desired) {
            continue;
        }

        create_new_prototype_callback(ctx.clone(), *desired).await?;
    }

    Ok(())
}
