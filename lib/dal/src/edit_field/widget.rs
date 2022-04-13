pub mod array_widget;
pub mod checkbox_widget;
pub mod header_widget;
pub mod map_widget;
pub mod select_widget;
pub mod text_widget;

use crate::edit_field::widget::map_widget::MapWidget;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumString};

pub use self::{
    array_widget::ArrayWidget,
    checkbox_widget::CheckboxWidget,
    header_widget::HeaderWidget,
    select_widget::{SelectWidget, ToSelectWidget},
    text_widget::TextWidget,
};

pub mod prelude {
    pub use super::{
        array_widget::ArrayWidget,
        checkbox_widget::CheckboxWidget,
        header_widget::HeaderWidget,
        map_widget::MapWidget,
        select_widget::{SelectWidget, ToSelectWidget},
        text_widget::TextWidget,
        Widget, WidgetKind,
    };
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "kind", content = "options")]
pub enum Widget {
    Array(ArrayWidget),
    Checkbox(CheckboxWidget),
    Header(HeaderWidget),
    Map(MapWidget),
    Select(SelectWidget),
    Text(TextWidget),
}

#[derive(AsRefStr, Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Display, EnumString)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum WidgetKind {
    Array,
    Checkbox,
    Header,
    Map,
    SecretSelect,
    Text,
}
