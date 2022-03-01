pub mod widget;

use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgTxn};
use std::{future::Future, pin::Pin};
use strum_macros::{AsRefStr, Display, EnumString};
use thiserror::Error;
use veritech::EncryptionKey;

use crate::{
    func::backend::validation::ValidationError, HistoryActor, LabelListError, PropId, PropKind,
    SystemId, Tenancy, Visibility,
};

pub use widget::{ToSelectWidget, Widget};

#[derive(Error, Debug)]
pub enum EditFieldError {
    #[error("invalid edit field name: {0}")]
    InvalidField(String),
    #[error("value is not expected type: {0}")]
    InvalidValueType(&'static str),
    #[error("label list error: {0}")]
    LabelList(#[from] LabelListError),
    #[error("value for edit field not provided, cannot set value")]
    MissingValue,
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("missing head value in visibility diff calculation")]
    VisibilityDiffMissingHeadValue,
    #[error("missing change set value in visibility diff calculation")]
    VisibilityDiffMissingChangeSetValue,
}

impl EditFieldError {
    pub fn invalid_field(field_name: impl Into<String>) -> Self {
        Self::InvalidField(field_name.into())
    }
}

pub type EditFieldResult<T> = Result<T, EditFieldError>;

// NOTE: This might not need to be its own thing. We might be able to use PropKind directly?
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum EditFieldDataType {
    Array,
    Boolean,
    Integer,
    Map,
    None,
    Object,
    String,
}

impl From<PropKind> for EditFieldDataType {
    fn from(prop_kind: PropKind) -> Self {
        match prop_kind {
            PropKind::Array => EditFieldDataType::Array,
            PropKind::Boolean => EditFieldDataType::Boolean,
            PropKind::Integer => EditFieldDataType::Integer,
            PropKind::Map => EditFieldDataType::Map,
            PropKind::Object => EditFieldDataType::Object,
            PropKind::String => EditFieldDataType::String,
        }
    }
}

#[derive(AsRefStr, Clone, Debug, Deserialize, Display, EnumString, Eq, PartialEq, Serialize)]
pub enum EditFieldObjectKind {
    /// Update the Component itself.
    Component,
    /// Update a property on the Component (and not the Component itself).
    ComponentProp,
    Prop,
    QualificationCheck,
    Schema,
    SchemaUiMenu,
    SchemaVariant,
    Socket,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "kind", content = "value")]
pub enum VisibilityDiff {
    None,
    Head(Option<serde_json::Value>),
    ChangeSet(Option<serde_json::Value>),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EditFieldBaggageComponentProp {
    pub prop_id: PropId,
    pub system_id: Option<SystemId>,
}

#[derive(AsRefStr, Clone, Debug, Deserialize, Display, Eq, PartialEq, Serialize)]
pub enum EditFieldBaggage {
    ComponentProp(EditFieldBaggageComponentProp),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct EditField {
    id: String,
    pub name: String,
    /// A descendant edit field can be specified using "path" (e.g. "metadata.name").
    path: Vec<String>,
    object_kind: EditFieldObjectKind,
    object_id: i64,
    data_type: EditFieldDataType,
    widget: Widget,
    pub value: Option<serde_json::Value>,
    visibility_diff: VisibilityDiff,
    validation_errors: Vec<ValidationError>,
    baggage: Option<EditFieldBaggage>,
}

impl EditField {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        path: Vec<String>,
        object_kind: EditFieldObjectKind,
        object_id: impl Into<i64>,
        data_type: EditFieldDataType,
        widget: Widget,
        value: Option<serde_json::Value>,
        visibility_diff: VisibilityDiff,
        validation_errors: Vec<ValidationError>,
    ) -> Self {
        let name = name.into();
        let object_id = object_id.into();
        let mut id_parts = path.clone();
        id_parts.push(name.clone());
        let id = id_parts.join(".");
        EditField {
            id,
            object_kind,
            object_id,
            data_type,
            widget,
            name,
            path,
            value,
            visibility_diff,
            validation_errors,
            baggage: None,
        }
    }

    pub fn set_baggage(&mut self, baggage: EditFieldBaggage) {
        self.baggage = Some(baggage);
    }

    pub fn unset_baggage(&mut self) {
        self.baggage = None;
    }
}

pub type EditFields = Vec<EditField>;

pub type UpdateFunction = Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = EditFieldResult<()>>>>>;

#[async_trait::async_trait]
pub trait EditFieldAble {
    type Id;
    type Error;

    async fn get_edit_fields(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        id: &Self::Id,
    ) -> Result<EditFields, Self::Error>;

    #[allow(clippy::too_many_arguments)]
    async fn update_from_edit_field(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        veritech: veritech::Client,
        encryption_key: &EncryptionKey,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        id: Self::Id,
        edit_field_id: String,
        value: Option<serde_json::Value>,
    ) -> Result<(), Self::Error>;
}

pub fn value_and_visibility_diff_option<Obj, Value: Eq + Serialize + ?Sized>(
    visibility: &Visibility,
    target_obj: Option<&Obj>,
    target_fn: impl Fn(&Obj) -> Option<&Value> + Copy,
    head_obj: Option<&Obj>,
    change_set_obj: Option<&Obj>,
) -> EditFieldResult<(Option<serde_json::Value>, VisibilityDiff)> {
    let target_value = target_obj.map(target_fn);
    let head_value_option = head_obj.map(target_fn);
    let change_set_value_option = change_set_obj.map(target_fn);
    let visibility_diff = visibility_diff(
        visibility,
        target_value.as_ref(),
        head_value_option.as_ref(),
        change_set_value_option.as_ref(),
    )?;
    let mut value = None;
    if let Some(target_value_real) = target_value {
        value = Some(serde_json::to_value(target_value_real)?);
    }
    Ok((value, visibility_diff))
}

pub fn value_and_visibility_diff_json_option<Obj>(
    visibility: &Visibility,
    target_obj: Option<&Obj>,
    target_fn: impl Fn(&Obj) -> Option<&serde_json::Value> + Copy,
    head_obj: Option<&Obj>,
    change_set_obj: Option<&Obj>,
) -> EditFieldResult<(Option<serde_json::Value>, VisibilityDiff)> {
    let target_value = target_obj.map(target_fn);
    let head_value_option = head_obj.map(target_fn);
    let change_set_value_option = change_set_obj.map(target_fn);
    let visibility_diff = visibility_diff(
        visibility,
        target_value.as_ref(),
        head_value_option.as_ref(),
        change_set_value_option.as_ref(),
    )?;
    let mut value = None;
    if let Some(target_value_real) = target_value {
        value = target_value_real.cloned();
    }
    Ok((value, visibility_diff))
}

pub fn value_and_visibility_diff<Obj, Value: Eq + Serialize + ?Sized>(
    visibility: &Visibility,
    target_obj: Option<&Obj>,
    target_fn: impl Fn(&Obj) -> &Value + Copy,
    head_obj: Option<&Obj>,
    change_set_obj: Option<&Obj>,
) -> EditFieldResult<(Option<serde_json::Value>, VisibilityDiff)> {
    let target_value = target_obj.map(target_fn);
    let head_value_option = head_obj.map(target_fn);
    let change_set_value_option = change_set_obj.map(target_fn);
    let visibility_diff = visibility_diff(
        visibility,
        target_value,
        head_value_option,
        change_set_value_option,
    )?;
    let mut value = None;
    if let Some(target_value_real) = target_value {
        value = Some(serde_json::to_value(target_value_real)?);
    }
    Ok((value, visibility_diff))
}

pub fn value_and_visibility_diff_copy<Obj, Value: Eq + Serialize>(
    visibility: &Visibility,
    target_obj: Option<&Obj>,
    target_fn: impl Fn(&Obj) -> Value + Copy,
    head_obj: Option<&Obj>,
    change_set_obj: Option<&Obj>,
) -> EditFieldResult<(Option<serde_json::Value>, VisibilityDiff)> {
    let target_value = target_obj.map(target_fn);
    let head_value_option = head_obj.map(target_fn);
    let change_set_value_option = change_set_obj.map(target_fn);
    let visibility_diff = visibility_diff(
        visibility,
        target_value.as_ref(),
        head_value_option.as_ref(),
        change_set_value_option.as_ref(),
    )?;
    let mut value = None;
    if let Some(target_value_real) = target_value {
        value = Some(serde_json::to_value(target_value_real)?);
    }
    Ok((value, visibility_diff))
}

fn visibility_diff<Value: Eq + Serialize + ?Sized>(
    visibility: &Visibility,
    target_value_option: Option<&Value>,
    head_value_option: Option<&Value>,
    change_set_value_option: Option<&Value>,
) -> EditFieldResult<VisibilityDiff> {
    let mut visibility_diff = VisibilityDiff::None;
    if visibility.in_change_set() {
        visibility_diff = match (target_value_option, head_value_option) {
            (Some(target_value), Some(head_value)) => {
                if target_value != head_value {
                    VisibilityDiff::Head(Some(serde_json::to_value(head_value)?))
                } else {
                    VisibilityDiff::None
                }
            }
            (Some(_), None) => VisibilityDiff::Head(None),
            (None, Some(head_value)) => {
                VisibilityDiff::Head(Some(serde_json::to_value(head_value)?))
            }
            (None, None) => VisibilityDiff::None,
        };
    }
    if visibility.in_edit_session() {
        visibility_diff = match (target_value_option, change_set_value_option) {
            (Some(target_value), Some(change_set_value)) => {
                if target_value != change_set_value {
                    VisibilityDiff::ChangeSet(Some(serde_json::to_value(change_set_value)?))
                } else {
                    visibility_diff
                }
            }
            (Some(_), None) => visibility_diff,
            (None, Some(change_set_value)) => {
                VisibilityDiff::ChangeSet(Some(serde_json::to_value(change_set_value)?))
            }
            (None, None) => visibility_diff,
        };
    }
    Ok(visibility_diff)
}
