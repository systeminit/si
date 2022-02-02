pub mod array_widget;
pub mod checkbox_widget;
pub mod header_widget;
pub mod select_widget;
pub mod text_widget;

use serde::{Deserialize, Serialize};

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
        select_widget::{SelectWidget, ToSelectWidget},
        text_widget::TextWidget,
        Widget,
    };
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, Clone)]
#[serde(tag = "kind", content = "options")]
pub enum Widget {
    Array(ArrayWidget),
    Checkbox(CheckboxWidget),
    Header(HeaderWidget),
    Select(SelectWidget),
    Text(TextWidget),
}
