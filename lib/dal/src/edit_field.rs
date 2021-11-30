use serde::{Deserialize, Serialize};
use si_data::{NatsTxn, PgTxn};
use std::{future::Future, pin::Pin};
use thiserror::Error;

use crate::{HistoryActor, LabelList, Tenancy, Visibility};

#[derive(Error, Debug)]
pub enum EditFieldError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("missing head value in visibility diff calculation")]
    VisibilityDiffMissingHeadValue,
    #[error("missing change set value in visibility diff calculation")]
    VisibilityDiffMissingChangeSetValue,
}

pub type EditFieldResult<T> = Result<T, EditFieldError>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum EditFieldDataType {
    String,
    Number,
    Object,
    Boolean,
    Map,
    Array,
    None,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "kind", content = "options")]
pub enum Widget {
    Text(TextWidget),
    Select(SelectWidget),
    Header(HeaderWidget),
    Array(ArrayWidget),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub enum EditFieldObjectKind {
    Schema,
    SchemaUiMenu,
    SchemaVariant,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "kind", content = "value")]
pub enum VisibilityDiff {
    None,
    Head(Option<serde_json::Value>),
    ChangeSet(Option<serde_json::Value>),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct EditField {
    id: String,
    name: String,
    path: Vec<String>,
    object_kind: EditFieldObjectKind,
    object_id: i64,
    data_type: EditFieldDataType,
    widget: Widget,
    value: Option<serde_json::Value>,
    visibility_diff: VisibilityDiff,
    validators: Vec<Validator>,
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
        validators: Vec<Validator>,
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
            validators,
        }
    }
}

pub type EditFields = Vec<EditField>;

pub type UpdateFunction = Box<dyn Fn(String) -> Pin<Box<dyn Future<Output = EditFieldResult<()>>>>>;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct TextWidget {}

impl TextWidget {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for TextWidget {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct SelectWidget {
    options: LabelList<serde_json::Value>,
    default: Option<serde_json::Value>,
}

impl SelectWidget {
    pub fn new(options: LabelList<serde_json::Value>, default: Option<serde_json::Value>) -> Self {
        SelectWidget { options, default }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct HeaderWidget {
    edit_fields: EditFields,
}

impl HeaderWidget {
    pub fn new(edit_fields: EditFields) -> Self {
        HeaderWidget { edit_fields }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct ArrayWidget {
    entries: Vec<EditFields>,
}

impl ArrayWidget {
    pub fn new(entries: Vec<EditFields>) -> Self {
        ArrayWidget { entries }
    }
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "kind")]
pub enum Validator {
    Required(RequiredValidator),
    Regex(RegexValidator),
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct RequiredValidator;

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
pub struct RegexValidator;

#[async_trait::async_trait]
pub trait EditFieldAble {
    type Id;
    type ErrorKind;

    async fn get_edit_fields(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        id: &Self::Id,
    ) -> Result<EditFields, Self::ErrorKind>;

    #[allow(clippy::too_many_arguments)]
    async fn update_from_edit_field(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        id: Self::Id,
        edit_field_id: String,
        value: Option<serde_json::Value>,
    ) -> Result<(), Self::ErrorKind>;
}

pub fn value_and_visiblity_diff_option<Obj, Value: Eq + Serialize + ?Sized>(
    visibility: &Visibility,
    target_obj: Option<&Obj>,
    target_fn: impl Fn(&Obj) -> Option<&Value>,
    head_obj: Option<&Obj>,
    change_set_obj: Option<&Obj>,
) -> EditFieldResult<(Option<serde_json::Value>, VisibilityDiff)> {
    let target_value = target_obj.as_deref().map(|o| target_fn(o));
    let head_value_option = head_obj.as_deref().map(|o| target_fn(o));
    let change_set_value_option = change_set_obj.as_deref().map(|o| target_fn(o));
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

pub fn value_and_visiblity_diff<Obj, Value: Eq + Serialize + ?Sized>(
    visibility: &Visibility,
    target_obj: Option<&Obj>,
    target_fn: impl Fn(&Obj) -> &Value,
    head_obj: Option<&Obj>,
    change_set_obj: Option<&Obj>,
) -> EditFieldResult<(Option<serde_json::Value>, VisibilityDiff)> {
    let target_value = target_obj.as_deref().map(|o| target_fn(o));
    let head_value_option = head_obj.as_deref().map(|o| target_fn(o));
    let change_set_value_option = change_set_obj.as_deref().map(|o| target_fn(o));
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
