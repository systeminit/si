use chrono::{DateTime, Utc};
use derive_builder::{Builder, UninitializedFieldError};
use object_tree::Hash;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use thiserror::Error;
use url::Url;

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

#[derive(
    Deserialize,
    Serialize,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
)]
#[serde(rename_all = "camelCase")]
pub enum FuncArgumentKind {
    Array,
    Boolean,
    Integer,
    Object,
    String,
    Map,
    Any,
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct FuncArgumentSpec {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub kind: FuncArgumentKind,
    #[builder(setter(into))]
    pub element_kind: Option<FuncArgumentKind>,
}

impl FuncArgumentSpec {
    pub fn builder() -> FuncArgumentSpecBuilder {
        FuncArgumentSpecBuilder::default()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, AsRefStr, Display, EnumIter, EnumString)]
#[serde(rename_all = "camelCase")]
pub enum FuncSpecBackendKind {
    JsAttribute,
    JsWorkflow,
    JsCommand,
    JsValidation,
    Json,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, AsRefStr, Display, EnumIter, EnumString)]
#[serde(rename_all = "camelCase")]
pub enum FuncSpecBackendResponseType {
    Array,
    Boolean,
    Integer,
    Map,
    Object,
    Qualification,
    CodeGeneration,
    Confirmation,
    String,
    Json,
    Validation,
    Workflow,
    Command,
}

pub type FuncUniqueId = Hash;

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct FuncSpec {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into, strip_option), default)]
    pub display_name: Option<String>,
    #[builder(setter(into, strip_option), default)]
    pub description: Option<String>,
    #[builder(setter(into))]
    pub handler: String,
    #[builder(setter(into))]
    pub code_base64: String,
    #[builder(setter(into))]
    pub backend_kind: FuncSpecBackendKind,
    #[builder(setter(into))]
    pub response_type: FuncSpecBackendResponseType,
    #[builder(setter(into))]
    pub hidden: bool,
    #[builder(field(type = "FuncUniqueId", build = "self.build_func_unique_id()"))]
    pub unique_id: FuncUniqueId,

    #[builder(setter(into, strip_option), default)]
    pub link: Option<Url>,

    #[builder(setter(each(name = "argument"), into), default)]
    pub arguments: Vec<FuncArgumentSpec>,
}

impl FuncSpec {
    #[must_use]
    pub fn builder() -> FuncSpecBuilder {
        FuncSpecBuilder::default()
    }
}

impl FuncSpecBuilder {
    #[allow(unused_mut)]
    pub fn try_link<V>(&mut self, value: V) -> Result<&mut Self, V::Error>
    where
        V: TryInto<Url>,
    {
        let converted: Url = value.try_into()?;
        Ok(self.link(converted))
    }

    fn build_func_unique_id(&self) -> Hash {
        // Not happy about all these clones and unwraps...
        let mut bytes = vec![];
        bytes.extend(self.name.clone().unwrap_or("".to_string()).as_bytes());
        bytes.extend(
            self.display_name
                .clone()
                .unwrap_or(Some("".to_string()))
                .unwrap_or("".to_string())
                .as_bytes(),
        );
        bytes.extend(
            self.description
                .clone()
                .unwrap_or(Some("".to_string()))
                .unwrap_or("".to_string())
                .as_bytes(),
        );
        bytes.extend(self.handler.clone().unwrap_or("".to_string()).as_bytes());
        bytes.extend(
            self.code_base64
                .clone()
                .unwrap_or("".to_string())
                .as_bytes(),
        );
        bytes.extend(
            self.backend_kind
                .unwrap_or(FuncSpecBackendKind::Json)
                .to_string()
                .as_bytes(),
        );
        bytes.extend(
            self.response_type
                .unwrap_or(FuncSpecBackendResponseType::Json)
                .to_string()
                .as_bytes(),
        );
        bytes.extend(&[self.hidden.unwrap_or(false).into()]);

        Hash::new(&bytes)
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SchemaSpec {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into))]
    pub category: String,
    #[builder(setter(into, strip_option), default)]
    pub category_name: Option<String>,

    #[builder(setter(each(name = "variant", into)), default)]
    pub variants: Vec<SchemaVariantSpec>,
}

impl SchemaSpec {
    #[must_use]
    pub fn builder() -> SchemaSpecBuilder {
        SchemaSpecBuilder::default()
    }

    #[allow(unused_mut)]
    pub fn try_variant<I>(&mut self, item: I) -> Result<&mut Self, I::Error>
    where
        I: TryInto<SchemaVariantSpec>,
    {
        let converted: SchemaVariantSpec = item.try_into()?;
        self.variants.extend(Some(converted));
        Ok(self)
    }
}

impl TryFrom<SchemaSpecBuilder> for SchemaSpec {
    type Error = SpecError;

    fn try_from(value: SchemaSpecBuilder) -> Result<Self, Self::Error> {
        value.build()
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct SchemaVariantSpec {
    #[builder(setter(into))]
    pub name: String,
    #[builder(setter(into, strip_option), default)]
    pub link: Option<Url>,
    #[builder(setter(into, strip_option), default)]
    pub color: Option<String>,

    #[builder(private, default = "Self::default_domain()")]
    pub domain: PropSpec,

    #[builder(setter(each(name = "leaf_function"), into), default)]
    pub leaf_functions: Vec<LeafFunctionSpec>,
}

impl SchemaVariantSpec {
    pub fn builder() -> SchemaVariantSpecBuilder {
        SchemaVariantSpecBuilder::default()
    }
}

impl SchemaVariantSpecBuilder {
    fn default_domain() -> PropSpec {
        PropSpec::Object {
            validations: None,
            name: "domain".to_string(),
            entries: vec![],
        }
    }

    #[allow(unused_mut)]
    pub fn try_link<V>(&mut self, value: V) -> Result<&mut Self, V::Error>
    where
        V: TryInto<Url>,
    {
        let converted: Url = value.try_into()?;
        Ok(self.link(converted))
    }

    #[allow(unused_mut)]
    pub fn prop(&mut self, item: impl Into<PropSpec>) -> &mut Self {
        let converted: PropSpec = item.into();
        match self.domain.get_or_insert_with(Self::default_domain) {
            PropSpec::Object { entries, .. } => entries.push(converted),
            invalid => unreachable!(
                "domain prop is an object but was found to be: {:?}",
                invalid
            ),
        };
        self
    }

    #[allow(unused_mut)]
    pub fn try_prop<I>(&mut self, item: I) -> Result<&mut Self, I::Error>
    where
        I: TryInto<PropSpec>,
    {
        let converted: PropSpec = item.try_into()?;
        Ok(self.prop(converted))
    }

    #[allow(unused_mut)]
    pub fn props(&mut self, value: Vec<PropSpec>) -> &mut Self {
        match self.domain.get_or_insert_with(Self::default_domain) {
            PropSpec::Object { entries, .. } => *entries = value,
            invalid => unreachable!(
                "domain prop is an object but was found to be: {:?}",
                invalid
            ),
        };
        self
    }
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Copy,
)]
#[serde(rename_all = "camelCase")]
pub enum LeafKind {
    CodeGeneration,
    Qualification,
    Confirmation,
}

#[derive(
    Debug,
    Serialize,
    Deserialize,
    Clone,
    PartialEq,
    Eq,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Copy,
)]
#[serde(rename_all = "camelCase")]
pub enum LeafInputLocation {
    Code,
    DeletedAt,
    Domain,
    Resource,
}

impl LeafInputLocation {
    pub fn try_from_arg_name(arg_name: &str) -> Result<Self, SpecError> {
        Ok(match arg_name {
            "domain" => LeafInputLocation::Domain,
            "code" => LeafInputLocation::Code,
            "resource" => LeafInputLocation::Resource,
            "deleted_at" => LeafInputLocation::DeletedAt,
            _ => {
                return Err(SpecError::LeafInputLocationConversionError(
                    arg_name.to_string(),
                ))
            }
        })
    }
}

#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[builder(build_fn(error = "SpecError"))]
pub struct LeafFunctionSpec {
    #[builder(setter(into))]
    pub func_unique_id: FuncUniqueId,

    #[builder(setter(into))]
    pub leaf_kind: LeafKind,

    #[builder(setter(into))]
    pub inputs: Vec<LeafInputLocation>,
}

impl LeafFunctionSpec {
    pub fn builder() -> LeafFunctionSpecBuilder {
        LeafFunctionSpecBuilder::default()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ValidationSpec {
    IntegerIsBetweenTwoIntegers {
        lower_bound: i64,
        upper_bound: i64,
    },
    StringEquals {
        expected: String,
    },
    StringHasPrefix {
        expected: String,
    },
    StringInStringArray {
        expected: Vec<String>,
        display_expected: bool,
    },
    StringIsValidIpAddr,
    StringIsHexColor,
    StringIsNotEmpty,
    CustomValidation {
        func_unique_id: Hash,
    },
}

impl ValidationSpec {
    pub fn builder() -> ValidationSpecBuilder {
        ValidationSpecBuilder::default()
    }
}

#[derive(
    Clone, Copy, Debug, Eq, Hash, PartialEq, EnumIter, EnumString, Display, Serialize, Deserialize,
)]
pub enum ValidationSpecKind {
    IntegerIsBetweenTwoIntegers,
    StringEquals,
    StringHasPrefix,
    StringInStringArray,
    StringIsValidIpAddr,
    StringIsHexColor,
    StringIsNotEmpty,
    CustomValidation,
}

#[derive(Clone, Debug, Default)]
pub struct ValidationSpecBuilder {
    kind: Option<ValidationSpecKind>,
    upper_bound: Option<i64>,
    lower_bound: Option<i64>,
    expected_string: Option<String>,
    expected_string_array: Option<Vec<String>>,
    display_expected: Option<bool>,
    func_unique_id: Option<Hash>,
}

impl ValidationSpecBuilder {
    pub fn kind(&mut self, kind: ValidationSpecKind) -> &mut Self {
        self.kind = Some(kind);
        self
    }

    pub fn upper_bound(&mut self, upper_bound: i64) -> &mut Self {
        self.upper_bound = Some(upper_bound);
        self
    }

    pub fn lower_bound(&mut self, lower_bound: i64) -> &mut Self {
        self.lower_bound = Some(lower_bound);
        self
    }

    pub fn expected_string(&mut self, expected_string: String) -> &mut Self {
        self.expected_string = Some(expected_string);
        self
    }

    pub fn expected_string_array(&mut self, expected_string_array: Vec<String>) -> &mut Self {
        self.expected_string_array = Some(expected_string_array);
        self
    }

    pub fn display_expected(&mut self, display_expected: bool) -> &mut Self {
        self.display_expected = Some(display_expected);
        self
    }

    pub fn func_unique_id(&mut self, func_unique_id: Hash) -> &mut Self {
        self.func_unique_id = Some(func_unique_id);
        self
    }

    pub fn build(&self) -> Result<ValidationSpec, SpecError> {
        Ok(match self.kind {
            Some(kind) => match kind {
                ValidationSpecKind::IntegerIsBetweenTwoIntegers => {
                    ValidationSpec::IntegerIsBetweenTwoIntegers {
                        lower_bound: self
                            .lower_bound
                            .ok_or(UninitializedFieldError::from("lower_bound"))?,
                        upper_bound: self
                            .upper_bound
                            .ok_or(UninitializedFieldError::from("lower_bound"))?,
                    }
                }
                ValidationSpecKind::StringEquals => ValidationSpec::StringEquals {
                    expected: self
                        .expected_string
                        .as_ref()
                        .ok_or(UninitializedFieldError::from("expected_string"))?
                        .to_string(),
                },
                ValidationSpecKind::StringHasPrefix => ValidationSpec::StringHasPrefix {
                    expected: self
                        .expected_string
                        .as_ref()
                        .ok_or(UninitializedFieldError::from("expected_string"))?
                        .to_string(),
                },
                ValidationSpecKind::StringInStringArray => ValidationSpec::StringInStringArray {
                    display_expected: self
                        .display_expected
                        .ok_or(UninitializedFieldError::from("display_expected"))?,
                    expected: self
                        .expected_string_array
                        .clone()
                        .ok_or(UninitializedFieldError::from("expected_string"))?,
                },
                ValidationSpecKind::StringIsValidIpAddr => ValidationSpec::StringIsValidIpAddr,
                ValidationSpecKind::StringIsHexColor => ValidationSpec::StringIsHexColor,
                ValidationSpecKind::StringIsNotEmpty => ValidationSpec::StringIsNotEmpty,
                ValidationSpecKind::CustomValidation => ValidationSpec::CustomValidation {
                    func_unique_id: self
                        .func_unique_id
                        .ok_or(UninitializedFieldError::from("func_unique_id"))?,
                },
            },
            None => {
                return Err(UninitializedFieldError::from("kind").into());
            }
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PropSpec {
    #[serde(rename_all = "camelCase")]
    String {
        name: String,
        validations: Option<Vec<ValidationSpec>>,
    },
    #[serde(rename_all = "camelCase")]
    Number {
        name: String,
        validations: Option<Vec<ValidationSpec>>,
    },
    #[serde(rename_all = "camelCase")]
    Boolean {
        name: String,
        validations: Option<Vec<ValidationSpec>>,
    },
    #[serde(rename_all = "camelCase")]
    Map {
        name: String,
        type_prop: Box<PropSpec>,
        validations: Option<Vec<ValidationSpec>>,
    },
    #[serde(rename_all = "camelCase")]
    Array {
        name: String,
        type_prop: Box<PropSpec>,
        validations: Option<Vec<ValidationSpec>>,
    },
    #[serde(rename_all = "camelCase")]
    Object {
        name: String,
        entries: Vec<PropSpec>,
        validations: Option<Vec<ValidationSpec>>,
    },
}

impl PropSpec {
    pub fn builder() -> PropSpecBuilder {
        PropSpecBuilder::default()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PropSpecKind {
    String,
    Number,
    Boolean,
    Map,
    Array,
    Object,
}

#[derive(Clone, Debug, Default)]
pub struct PropSpecBuilder {
    kind: Option<PropSpecKind>,
    name: Option<String>,
    type_prop: Option<PropSpec>,
    entries: Vec<PropSpec>,
    validations: Vec<ValidationSpec>,
}

impl PropSpecBuilder {
    #[allow(unused_mut)]
    pub fn kind(&mut self, value: PropSpecKind) -> &mut Self {
        self.kind = Some(value);
        self
    }

    pub fn get_kind(&self) -> Option<PropSpecKind> {
        self.kind
    }

    #[allow(unused_mut)]
    pub fn name(&mut self, value: impl Into<String>) -> &mut Self {
        self.name = Some(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn type_prop(&mut self, value: impl Into<PropSpec>) -> &mut Self {
        self.type_prop = Some(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn entry(&mut self, value: impl Into<PropSpec>) -> &mut Self {
        self.entries.push(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn validation(&mut self, value: impl Into<ValidationSpec>) -> &mut Self {
        self.validations.push(value.into());
        self
    }

    #[allow(unused_mut)]
    pub fn entries(&mut self, value: Vec<impl Into<PropSpec>>) -> &mut Self {
        self.entries = value.into_iter().map(Into::into).collect();
        self
    }

    /// Builds a new `Prop`.
    ///
    /// # Errors
    ///
    /// If a required field has not been initialized.
    pub fn build(&self) -> Result<PropSpec, SpecError> {
        let name = match self.name {
            Some(ref name) => name.clone(),
            None => {
                return Err(UninitializedFieldError::from("name").into());
            }
        };

        let validations = self.validations.clone();

        Ok(match self.kind {
            Some(kind) => match kind {
                PropSpecKind::String => PropSpec::String {
                    name,
                    validations: Some(validations),
                },
                PropSpecKind::Number => PropSpec::Number {
                    name,
                    validations: Some(validations),
                },
                PropSpecKind::Boolean => PropSpec::Boolean {
                    name,
                    validations: Some(validations),
                },
                PropSpecKind::Map => PropSpec::Map {
                    name,
                    type_prop: match self.type_prop {
                        Some(ref value) => Box::new(value.clone()),
                        None => {
                            return Err(UninitializedFieldError::from("type_prop").into());
                        }
                    },
                    validations: Some(validations),
                },
                PropSpecKind::Array => PropSpec::Array {
                    name,
                    type_prop: match self.type_prop {
                        Some(ref value) => Box::new(value.clone()),
                        None => {
                            return Err(UninitializedFieldError::from("type_prop").into());
                        }
                    },
                    validations: Some(validations),
                },
                PropSpecKind::Object => PropSpec::Object {
                    name,
                    entries: self.entries.clone(),
                    validations: Some(validations),
                },
            },
            None => {
                return Err(UninitializedFieldError::from("kind").into());
            }
        })
    }
}

impl TryFrom<PropSpecBuilder> for PropSpec {
    type Error = SpecError;

    fn try_from(value: PropSpecBuilder) -> Result<Self, Self::Error> {
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
