use chrono::{DateTime, Utc};
use derive_builder::{Builder, UninitializedFieldError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod attr_func_input;
mod func;
mod func_description;
mod leaf_function;
mod prop;
mod schema;
mod socket;
mod validation;
mod variant;
mod workflow;

pub use {
    attr_func_input::*, func::*, func_description::*, leaf_function::*, prop::*, schema::*,
    socket::*, validation::*, variant::*, workflow::*,
};

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct PkgSpec {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub version: String,

    #[builder(setter(into), default)]
    pub description: String,
    #[builder(try_setter, setter(into), default = "Utc::now()")]
    pub created_at: DateTime<Utc>,
    #[builder(setter(into))]
    pub created_by: String,

    #[builder(setter(each(name = "schema", into)), default)]
    pub schemas: Vec<SchemaSpec>,

    #[builder(setter(each(name = "func", into)), default)]
    pub funcs: Vec<FuncSpec>,
}

impl PkgSpec {
    pub fn builder() -> PkgSpecBuilder {
        PkgSpecBuilder::default()
    }

    pub fn func_for_unique_id(&self, unique_id: &FuncUniqueId) -> Option<&FuncSpec> {
        self.funcs
            .iter()
            .find(|func_spec| &func_spec.unique_id == unique_id)
    }

    pub fn func_for_name(&self, name: impl AsRef<str>) -> Option<&FuncSpec> {
        let name = name.as_ref();

        self.funcs
            .iter()
            .find(|func_spec| func_spec.name.as_str() == name)
    }
}

impl PkgSpecBuilder {
    #[allow(unused_mut)]
    pub fn try_schema<I>(&mut self, item: I) -> Result<&mut Self, I::Error>
    where
        I: TryInto<SchemaSpec>,
    {
        let converted: SchemaSpec = item.try_into()?;
        Ok(self.schema(converted))
    }

    #[allow(unused_mut)]
    pub fn try_func<I>(&mut self, item: I) -> Result<&mut Self, I::Error>
    where
        I: TryInto<FuncSpec>,
    {
        let converted: FuncSpec = item.try_into()?;
        Ok(self.func(converted))
    }
}

impl TryFrom<PkgSpecBuilder> for PkgSpec {
    type Error = SpecError;

    fn try_from(value: PkgSpecBuilder) -> Result<Self, Self::Error> {
        value.build()
    }
}

#[derive(Debug, Error)]
pub enum SpecError {
    /// Uninitialized field
    #[error("{0} must be initialized")]
    UninitializedField(&'static str),
    /// Custom validation error
    #[error("{0}")]
    ValidationError(String),
    #[error("Can't convert {0} to LeafInputLocation")]
    LeafInputLocationConversionError(String),
}

impl From<UninitializedFieldError> for SpecError {
    fn from(value: UninitializedFieldError) -> Self {
        Self::UninitializedField(value.field_name())
    }
}

impl From<String> for SpecError {
    fn from(value: String) -> Self {
        Self::ValidationError(value)
    }
}
