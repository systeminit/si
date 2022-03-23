use async_trait::async_trait;
use jwt_simple::reexports::serde_json::Value;
use serde::{Deserialize, Serialize};

use si_data::{NatsTxn, PgTxn};
use telemetry::prelude::*;
use veritech::EncryptionKey;

use crate::{
    edit_field::{
        value_and_visibility_diff, value_and_visibility_diff_option, widget::prelude::*, EditField,
        EditFieldAble, EditFieldDataType, EditFieldError, EditFieldObjectKind, EditFields,
    },
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    standard_model_many_to_many, HistoryActor, LabelList, ReadTenancy, SchemaError, SchematicKind,
    StandardModel, Tenancy, Timestamp, Visibility, WriteTenancy,
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
    tenancy: Tenancy,
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
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        write_tenancy: &WriteTenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        schematic_kind: &SchematicKind,
    ) -> SchemaResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM schema_ui_menu_create_v1($1, $2, $3)",
                &[write_tenancy, &visibility, &schematic_kind.as_ref()],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            &write_tenancy.into(),
            visibility,
            history_actor,
            row,
        )
        .await?;
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
    pub async fn usable_in_menu(
        &self,
        txn: &PgTxn<'_>,
        visibility: &Visibility,
    ) -> SchemaResult<bool> {
        if self.name().is_none()
            || self.category().is_none()
            || self.schema(txn, visibility).await?.is_none()
        {
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
        txn: &PgTxn<'_>,
        read_tenancy: &ReadTenancy,
        visibility: &Visibility,
        id: &Self::Id,
    ) -> Result<EditFields, Self::Error> {
        let object = UiMenu::get_by_id(txn, &read_tenancy.into(), visibility, id)
            .await?
            .ok_or(SchemaError::UiMenuNotFound(*id))?;
        let head_obj: Option<UiMenu> = if visibility.in_change_set() {
            let head_visibility = Visibility::new_head(visibility.deleted);
            UiMenu::get_by_id(txn, &read_tenancy.into(), &head_visibility, id).await?
        } else {
            None
        };
        let change_set_obj: Option<UiMenu> = if visibility.in_change_set() {
            let change_set_visibility =
                Visibility::new_change_set(visibility.change_set_pk, visibility.deleted);
            UiMenu::get_by_id(txn, &read_tenancy.into(), &change_set_visibility, id).await?
        } else {
            None
        };
        let (name_value, name_visibility_diff) = value_and_visibility_diff_option(
            visibility,
            Some(&object),
            UiMenu::name,
            head_obj.as_ref(),
            change_set_obj.as_ref(),
        )?;
        let (category_value, category_visibility_diff) = value_and_visibility_diff_option(
            visibility,
            Some(&object),
            UiMenu::category,
            head_obj.as_ref(),
            change_set_obj.as_ref(),
        )?;
        let (schematic_kind_value, schematic_kind_visibility_diff) = value_and_visibility_diff(
            visibility,
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
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        _veritech: veritech::Client,
        _encryption_key: &EncryptionKey,
        write_tenancy: &WriteTenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        id: Self::Id,
        edit_field_id: String,
        value: Option<Value>,
    ) -> Result<(), Self::Error> {
        let edit_field_id = edit_field_id.as_ref();
        let mut object = UiMenu::get_by_id(txn, &write_tenancy.into(), visibility, &id)
            .await?
            .ok_or(SchemaError::UiMenuNotFound(id))?;
        match edit_field_id {
            // TODO: Pretty sure this can handle the case where we want to discard the value,
            // since it was optional in the first place.
            "name" => match value {
                Some(json_value) => {
                    let value = json_value.as_str().map(|s| s.to_string());
                    object
                        .set_name(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            "category" => match value {
                Some(json_value) => {
                    let value = json_value.as_str().map(|s| s.to_string());
                    object
                        .set_category(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            "schematic_kind" => match value {
                Some(json_value) => {
                    let value: SchematicKind = serde_json::from_value(json_value)
                        .expect("value must be a string, and it aint");
                    object
                        .set_schematic_kind(txn, nats, visibility, history_actor, value)
                        .await?;
                }
                None => return Err(EditFieldError::MissingValue.into()),
            },
            invalid => return Err(EditFieldError::invalid_field(invalid).into()),
        }
        Ok(())
    }
}
