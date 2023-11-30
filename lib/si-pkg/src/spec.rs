use chrono::{DateTime, Utc};
use derive_builder::{Builder, UninitializedFieldError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod action_func;
mod attr_func_input;
mod attribute_value;
mod authentication_func;
mod change_set;
mod component;
mod edge;
mod func;
mod leaf_function;
mod map_key_func;
mod position;
mod prop;
mod root_prop_func;
mod schema;
mod si_prop_func;
mod socket;
mod validation;
mod variant;

pub use {
    action_func::*, attr_func_input::*, attribute_value::*, authentication_func::*, change_set::*,
    component::*, edge::*, func::*, leaf_function::*, map_key_func::*, position::*, prop::*,
    root_prop_func::*, schema::*, si_prop_func::*, socket::*, validation::*, variant::*,
};

use super::SiPkgKind;

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct PkgSpec {
    #[builder(setter(into), default = "SiPkgKind::Module")]
    pub kind: SiPkgKind,
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
    #[builder(setter(into, strip_option), default)]
    #[serde(default)]
    pub default_change_set: Option<String>,
    #[builder(setter(into, strip_option), default)]
    #[serde(default)]
    pub workspace_pk: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub workspace_name: Option<String>,

    #[builder(setter(each(name = "schema", into)), default)]
    #[serde(default)]
    pub schemas: Vec<SchemaSpec>,

    #[builder(setter(each(name = "func", into)), default)]
    #[serde(default)]
    pub funcs: Vec<FuncSpec>,

    #[builder(setter(each(name = "change_set", into)), default)]
    #[serde(default)]
    pub change_sets: Vec<ChangeSetSpec>,
}

impl PkgSpec {
    pub fn builder() -> PkgSpecBuilder {
        PkgSpecBuilder::default()
    }

    pub fn func_for_unique_id(&self, unique_id: &str) -> Option<&FuncSpec> {
        self.funcs
            .iter()
            .find(|func_spec| func_spec.unique_id == unique_id)
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

#[remain::sorted]
#[derive(Debug, Error)]
pub enum SpecError {
    #[error("Can't convert {0} to LeafInputLocation")]
    LeafInputLocationConversionError(String),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    /// Uninitialized field
    #[error("{0} must be initialized")]
    UninitializedField(&'static str),
    /// Custom validation error
    #[error("{0}")]
    ValidationError(String),
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
