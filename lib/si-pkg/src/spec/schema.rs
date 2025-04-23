use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    SchemaVariantSpec,
    SpecError,
};

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SchemaSpecData {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub category: String,
    #[builder(setter(into, strip_option), default)]
    pub category_name: Option<String>,
    #[builder(setter(into), default)]
    pub ui_hidden: bool,
    #[builder(setter(into, strip_option), default)]
    pub default_schema_variant: Option<String>,
}

impl SchemaSpecData {
    #[must_use]
    pub fn builder() -> SchemaSpecDataBuilder {
        SchemaSpecDataBuilder::default()
    }

    pub fn anonymize(&mut self) {
        self.default_schema_variant = None;
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SchemaSpec {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into, strip_option), default)]
    pub data: Option<SchemaSpecData>,
    #[builder(setter(into, strip_option), default)]
    #[serde(default)]
    pub unique_id: Option<String>,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub deleted: bool,
    #[builder(setter(into), default)]
    #[serde(default)]
    pub is_builtin: bool,

    #[builder(setter(each(name = "variant", into)), default)]
    pub variants: Vec<SchemaVariantSpec>,
}

impl SchemaSpec {
    #[must_use]
    pub fn builder() -> SchemaSpecBuilder {
        SchemaSpecBuilder::default()
    }

    pub fn anonymize(&mut self) {
        self.unique_id = None;

        if let Some(ref mut data) = self.data {
            data.anonymize();
        }

        self.variants.iter_mut().for_each(|f| f.anonymize());
    }
}

impl TryFrom<SchemaSpecBuilder> for SchemaSpec {
    type Error = SpecError;

    fn try_from(value: SchemaSpecBuilder) -> Result<Self, Self::Error> {
        value.build()
    }
}
