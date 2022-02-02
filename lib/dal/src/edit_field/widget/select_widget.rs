use serde::{Deserialize, Serialize};

use crate::{edit_field::EditFieldResult, label_list::ToLabelList, LabelList};

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

pub trait ToSelectWidget: ToLabelList {
    fn to_select_widget_with_no_default() -> EditFieldResult<SelectWidget> {
        Ok(SelectWidget::new(Self::to_label_list()?, None))
    }

    fn to_select_widget_with<D>(default: D) -> EditFieldResult<SelectWidget>
    where
        D: Serialize,
    {
        Ok(SelectWidget::new(
            Self::to_label_list()?,
            Some(serde_json::to_value(default)?),
        ))
    }
}

pub trait ToSelectWidgetDefault: ToLabelList + Default {
    fn to_select_widget_with_default() -> EditFieldResult<SelectWidget> {
        Ok(SelectWidget::new(
            Self::to_label_list()?,
            Some(serde_json::to_value(Self::default())?),
        ))
    }
}
