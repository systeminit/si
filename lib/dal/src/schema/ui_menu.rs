use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    DalContext, DiagramKind, StandardModel, Timestamp, Visibility, WriteTenancy,
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
    diagram_kind: DiagramKind,
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
    pub async fn new(ctx: &DalContext, diagram_kind: &DiagramKind) -> SchemaResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM schema_ui_menu_create_v1($1, $2, $3)",
                &[
                    ctx.write_tenancy(),
                    ctx.visibility(),
                    &diagram_kind.as_ref(),
                ],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(name, Option<String>, SchemaResult);
    standard_model_accessor!(category, Option<String>, SchemaResult);
    standard_model_accessor!(diagram_kind, Enum(DiagramKind), SchemaResult);

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

    /// A menu item is unusable when it doesn't have the fields set yet that
    /// enable it to show up in a menu.
    pub async fn usable_in_menu(&self, ctx: &DalContext) -> SchemaResult<bool> {
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
