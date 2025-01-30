use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{
    management::prototype::{ManagementPrototype, ManagementPrototypeId},
    DalContext, Func, FuncId, Prop, Schema, SchemaId, SchemaVariant, SchemaVariantId, SocketArity,
};

use super::{EventualParent, FuncBinding, FuncBindingResult};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ManagementBinding {
    pub schema_variant_id: SchemaVariantId,
    pub management_prototype_id: ManagementPrototypeId,
    pub func_id: FuncId,
    pub managed_schemas: Option<Vec<SchemaId>>,
}

const INCOMING_CONNECTION_TYPE: &str = "{ component: string, socket: string; value: any }";
const DEFAULT_THIS_COMPONENT_IFACE: &str = "object";
const DEFAULT_THIS_INCOMING_CONNECTIONS: &str = "object";
const DEFAULT_COMPONENT_TYPES: &str = "object";

impl ManagementBinding {
    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.action.create_management_binding"
    )]
    pub async fn create_management_binding(
        ctx: &DalContext,
        func_id: FuncId,
        schema_variant_id: SchemaVariantId,
        managed_schemas: Option<HashSet<SchemaId>>,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        // don't add binding if parent is locked
        SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;

        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        ManagementPrototype::new(
            ctx,
            func.name.to_owned(),
            func.description.to_owned(),
            func.id,
            managed_schemas,
            schema_variant_id,
        )
        .await?;

        FuncBinding::for_func_id(ctx, func_id).await
    }

    pub async fn compile_management_types(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<String> {
        let default_types = (
            DEFAULT_THIS_COMPONENT_IFACE.to_owned(),
            DEFAULT_THIS_INCOMING_CONNECTIONS.to_owned(),
            DEFAULT_COMPONENT_TYPES.to_owned(),
            DEFAULT_COMPONENT_TYPES.to_owned(),
        );

        let (
            this_component_iface,
            this_incoming_connections,
            component_create_type,
            component_input_type,
        ) = match ManagementPrototype::prototype_id_for_func_id(ctx, func_id).await? {
            Some(prototype_id) => match ManagementPrototype::get_by_id(ctx, prototype_id).await? {
                Some(prototype) => {
                    let variant_id =
                        ManagementPrototype::get_schema_variant_id(ctx, prototype_id).await?;
                    let root_prop = Prop::get_by_id(
                        ctx,
                        SchemaVariant::get_root_prop_id(ctx, variant_id).await?,
                    )
                    .await?;

                    let (_, reverse_map) = prototype.managed_schemas_map(ctx).await?;

                    let mut component_create_types = vec![];
                    let mut component_update_types = vec![];
                    for (schema_id, name) in reverse_map {
                        let variant_id =
                            Schema::get_or_install_default_variant(ctx, schema_id).await?;

                        let root_prop = Prop::get_by_id(
                            ctx,
                            SchemaVariant::get_root_prop_id(ctx, variant_id).await?,
                        )
                        .await?;

                        let sv_type = root_prop.ts_type(ctx).await?;

                        let component_update_type = format!(
                            r#"
                                {{
                                    kind: "{name}",
                                    properties?: {sv_type},
                                    geometry?: {{ [key: string]: Geometry }},
                                    connect?: {{
                                        from: string,
                                        to: {{
                                            component: string;
                                            socket: string;
                                        }}
                                    }}[],
                                    parent?: string,
                                }}
                                "#
                        );

                        let component_create_type = format!(
                            r#"
                                {{
                                    kind?: "{name}",
                                    properties?: {sv_type},
                                    geometry?: Geometry | {{ [key: string]: Geometry }},
                                    connect?: Connection[],
                                    parent?: string,
                                }}
                                "#
                        );
                        component_create_types.push(component_create_type);
                        component_update_types.push(component_update_type);
                    }

                    let component_create_type = component_create_types.join("|\n");
                    let component_input_type = component_update_types.join("|\n");

                    let mut this_incoming_connections = "    {\n".to_string();
                    let this_component_iface = root_prop.ts_type(ctx).await?;
                    for input_socket in SchemaVariant::list_all_sockets(ctx, variant_id).await?.1 {
                        let name = input_socket.name();
                        let type_qualifier = match input_socket.arity() {
                            SocketArity::One => " | undefined",
                            SocketArity::Many => "[]",
                        };
                        this_incoming_connections.push_str(&format!(
                            "      {name}: {INCOMING_CONNECTION_TYPE}{type_qualifier},\n"
                        ));
                    }
                    this_incoming_connections.push_str("    }\n");

                    (
                        this_component_iface,
                        this_incoming_connections,
                        component_create_type,
                        component_input_type,
                    )
                }
                None => default_types,
            },
            None => default_types,
        };

        Ok(format!(
            r#"
type Geometry = {{
    x?: number,
    y?: number,
    width?: number,
    height?: number,
}};
type Connection =
    | {{
        from: string,
        to: {{
            component: string,
            socket: string,
        }},
    }}
    | {{
        from: {{
            component: string,
            socket: string,
        }},
        to: string,
    }};
type Output = {{
  status: 'ok' | 'error';
  ops?: {{
    views?: {{ create?: string[]; remove?: string[] }},
    create?: {{ [key: string]: {component_create_type} }},
    update?: {{ [key: string]: {{ 
        properties?: {{ [key: string]: unknown }}, 
        geometry?: {{ [key: string]: Geometry }}, 
        connect?: {{
            add?: Connection[],
            remove?: Connection[],
        }},
        parent?: string,
    }} }},
    delete?: string[],
    erase?: string[],
    remove?: {{ [key: string]: string[] }},
    actions?: {{ [key: string]: {{
      add?: ("create" | "update" | "refresh" | "delete" | string)[];
      remove?: ("create" | "update" | "refresh" | "delete" | string)[];
    }} }}
  }},
  message?: string | null;
}};
type Input = {{
  currentView: string, 
  thisComponent: {{
    properties: {this_component_iface},
    geometry: {{ [key: string]: Geometry }},
    incomingConnections: {this_incoming_connections},
  }},
  components: {{ [key: string]: {component_input_type} }},
  variantSocketMap: Record<string, number>,
}};"#
        ))
    }

    pub async fn assemble_management_bindings(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let mut bindings = vec![];
        for management_prototype_id in
            ManagementPrototype::list_ids_for_func_id(ctx, func_id).await?
        {
            let Some(prototype) =
                ManagementPrototype::get_by_id(ctx, management_prototype_id).await?
            else {
                error!("Could not get bindings for func_id {func_id}");
                continue;
            };

            let managed_schemas = prototype
                .managed_schemas()
                .map(|schemas| schemas.iter().map(ToOwned::to_owned).collect());
            let schema_variant_id =
                ManagementPrototype::get_schema_variant_id(ctx, management_prototype_id).await;
            match schema_variant_id {
                Ok(schema_variant_id) => {
                    bindings.push(FuncBinding::Management(ManagementBinding {
                        schema_variant_id,
                        func_id,
                        management_prototype_id,
                        managed_schemas,
                    }));
                }
                Err(err) => {
                    error!(error=?err, "Could not get bindings for func_id {func_id}");
                }
            }
        }

        Ok(bindings)
    }

    pub async fn port_binding_to_new_func(
        &self,
        ctx: &DalContext,
        new_func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let schema_variant_id = self.schema_variant_id;

        let managed_schemas = ManagementPrototype::get_by_id(ctx, self.management_prototype_id)
            .await?
            .and_then(|prototype| prototype.managed_schemas().map(ToOwned::to_owned));

        ManagementPrototype::remove(ctx, self.management_prototype_id).await?;

        Self::create_management_binding(ctx, new_func_id, schema_variant_id, managed_schemas)
            .await?;

        FuncBinding::for_func_id(ctx, new_func_id).await
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.action.delete_action_binding"
    )]
    pub async fn delete_management_binding(
        ctx: &DalContext,
        management_prototype_id: ManagementPrototypeId,
    ) -> FuncBindingResult<EventualParent> {
        // don't delete binding if parent is locked
        let schema_variant_id =
            ManagementPrototype::get_schema_variant_id(ctx, management_prototype_id).await?;
        SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;

        ManagementPrototype::remove(ctx, management_prototype_id).await?;

        Ok(EventualParent::SchemaVariant(schema_variant_id))
    }

    pub async fn update_management_binding(
        ctx: &DalContext,
        management_prototype_id: ManagementPrototypeId,
        managed_schemas: Option<Vec<SchemaId>>,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let func_id = ManagementPrototype::func_id(ctx, management_prototype_id).await?;
        let Some(management_prototype) =
            ManagementPrototype::get_by_id(ctx, management_prototype_id).await?
        else {
            return FuncBinding::for_func_id(ctx, func_id).await;
        };

        let schema_variant_id =
            ManagementPrototype::get_schema_variant_id(ctx, management_prototype_id).await?;
        let eventual_parent = EventualParent::SchemaVariant(schema_variant_id);
        eventual_parent.error_if_locked(ctx).await?;
        let manager_schema_id =
            SchemaVariant::schema_id_for_schema_variant_id(ctx, schema_variant_id).await?;

        management_prototype
            .modify(ctx, |proto| {
                proto.managed_schemas = managed_schemas.map(|schemas| {
                    schemas
                        .into_iter()
                        .filter(|schema_id| schema_id != &manager_schema_id)
                        .collect()
                });
                Ok(())
            })
            .await?;

        FuncBinding::for_func_id(ctx, func_id).await
    }
}
