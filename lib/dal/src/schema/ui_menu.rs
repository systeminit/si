use crate::DalContext;
use async_trait::async_trait;
use jwt_simple::reexports::serde_json::Value;
use serde::{Deserialize, Serialize};

use telemetry::prelude::*;

use crate::{
    edit_field::{
        value_and_visibility_diff, value_and_visibility_diff_option, widget::prelude::*, EditField,
        EditFieldAble, EditFieldDataType, EditFieldError, EditFieldObjectKind, EditFields,
    },
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    standard_model_many_to_many, LabelList, SchemaError, SchematicKind, StandardModel, Timestamp,
    Visibility, WriteTenancy,
};

use super::{Schema, SchemaId, SchemaResult};

pk!(UiMenuPk);
pk!(UiMenuId);

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct UiMenu {
    pub pk: UiMenuPk,
    id: UiMenuId,
    name: Option<String>,
    category: Option<String>,
    schematic_kind: SchematicKind,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: UiMenu,
    pk: UiMenuPk,
    id: UiMenuId,
    table_name: "schema_ui_menus",
    history_event_label_base: "schema_ui_menu",
    history_event_message_name: "Schema UI Menu"
}

impl UiMenu {
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        schematic_kind: &SchematicKind,
    ) -> SchemaResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM schema_ui_menu_create_v1($1, $2, $3)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &schematic_kind.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, Option<String>, SchemaResult);
    standard_model_accessor!(category, Option<String>, SchemaResult);
    standard_model_accessor!(schematic_kind, Enum(SchematicKind), SchemaResult);

    standard_model_belongs_to!(
        lookup_fn: schema,
        set_fn: set_schema,
        unset_fn: unset_schema,
        table: "schema_ui_menu_belongs_to_schema",
        model_table: "schemas",
        belongs_to_id: SchemaId,
        returns: Schema,
        result: SchemaResult,
    );

    standard_model_many_to_many!(
        lookup_fn: root_schematics,
        associate_fn: add_root_schematic,
        disassociate_fn: remove_root_schematic,
        table_name: "schema_ui_menu_root_schematic_many_to_many_schematic",
        left_table: "schema_ui_menus",
        left_id: UiMenuId,
        right_table: "schemas",
        right_id: SchemaId,
        which_table_is_this: "left",
        returns: Schema,
        result: SchemaResult,
    );

    /// A menu item is unusable when it doesn't have the fields set yet that
    /// enable it to show up in a menu.
    pub async fn usable_in_menu(&self, ctx: &DalContext<'_, '_>) -> SchemaResult<bool> {
        if self.name().is_none() || self.category().is_none() || self.schema(ctx).await?.is_none() {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    pub fn category_path(&self) -> Vec<String> {
        match self.category() {
            Some(category) => category.split('.').map(|f| f.to_string()).collect(),
            None => Vec::new(),
        }
    }
}

#[async_trait]
impl EditFieldAble for UiMenu {
    type Id = UiMenuId;
    type Error = SchemaError;

    async fn get_edit_fields(
        ctx: &DalContext<'_, '_>,
        id: &Self::Id,
    ) -> Result<EditFields, Self::Error> {
        let object = UiMenu::get_by_id(ctx, id)
            .await?
            .ok_or(SchemaError::UiMenuNotFound(*id))?;
        let head_obj: Option<UiMenu> = if ctx.visibility().in_change_set() {
            let _head_visibility = Visibility::new_head(ctx.visibility().deleted);
            UiMenu::get_by_id(ctx, id).await?
        } else {
            None
        };
        let change_set_obj: Option<UiMenu> = if ctx.visibility().in_change_set() {
            let _change_set_visibility = Visibility::new_change_set(
                ctx.visibility().change_set_pk,
                ctx.visibility().deleted,
            );
            UiMenu::get_by_id(ctx, id).await?
        } else {
            None
        };
        let (name_value, name_visibility_diff) = value_and_visibility_diff_option(
            ctx.visibility(),
            Some(&object),
            UiMenu::name,
            head_obj.as_ref(),
            change_set_obj.as_ref(),
        )?;
        let (category_value, category_visibility_diff) = value_and_visibility_diff_option(
            ctx.visibility(),
            Some(&object),
            UiMenu::category,
            head_obj.as_ref(),
            change_set_obj.as_ref(),
        )?;
        let (schematic_kind_value, schematic_kind_visibility_diff) = value_and_visibility_diff(
            ctx.visibility(),
            Some(&object),
            UiMenu::schematic_kind,
            head_obj.as_ref(),
            change_set_obj.as_ref(),
        )?;

        Ok(vec![
            EditField::new(
                String::from("name"),
                vec![],
                EditFieldObjectKind::SchemaUiMenu,
                object.id,
                EditFieldDataType::String,
                Widget::Text(TextWidget::new()),
                name_value,
                name_visibility_diff,
                vec![], // TODO: actually validate to generate ValidationErrors
            ),
            EditField::new(
                String::from("category"),
                vec![],
                EditFieldObjectKind::SchemaUiMenu,
                object.id,
                EditFieldDataType::String,
                Widget::Text(TextWidget::new()),
                category_value,
                category_visibility_diff,
                vec![], // TODO: actually validate to generate ValidationErrors
            ),
            EditField::new(
                String::from("schematic_kind"),
                vec![],
                EditFieldObjectKind::SchemaUiMenu,
                object.id,
                EditFieldDataType::String,
                Widget::Select(SelectWidget::new(
                    LabelList::new(vec![]),
                    Some(serde_json::to_value(object.schematic_kind)?),
                )),
                schematic_kind_value,
                schematic_kind_visibility_diff,
                vec![], // TODO: actually validate to generate ValidationErrors
            ),
        ])
    }

    async fn update_from_edit_field(
        ctx: &DalContext<'_, '_>,
        id: Self::Id,
        edit_field_id: String,
        value: Option<Value>,
    ) -> Result<(), Self::Error> {
        let edit_field_id = edit_field_id.as_ref();
        let mut object = UiMenu::get_by_id(ctx, &id)
            .await?
            .ok_or(SchemaError::UiMenuNotFound(id))?;
        match edit_field_id {
            // TODO: Pretty sure this can handle the case where we want to discard the value,
            // since it was optional in the first place.
            "name" => match value {
                Some(json_value) => {
                    let value = json_value.as_str().map(|s| s.to_string());
                    object.set_name(ctx, value).await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            "category" => match value {
                Some(json_value) => {
                    let value = json_value.as_str().map(|s| s.to_string());
                    object.set_category(ctx, value).await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            "schematic_kind" => match value {
                Some(json_value) => {
                    let value: SchematicKind = serde_json::from_value(json_value)
                        .expect("value must be a string, and it aint");
                    object.set_schematic_kind(ctx, value).await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            invalid => return Err(EditFieldError::invalid_field(invalid).into()),
        }
        Ok(())
    }
}
