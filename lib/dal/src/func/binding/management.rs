use std::collections::HashSet;

use itertools::Itertools;
use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;

use super::{
    EventualParent,
    FuncBinding,
    FuncBindingResult,
};
use crate::{
    DalContext,
    Func,
    FuncId,
    Prop,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    SocketArity,
    cached_module::CachedModule,
    management::prototype::{
        ManagementPrototype,
        ManagementPrototypeId,
    },
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ManagementBinding {
    pub schema_variant_id: SchemaVariantId,
    pub management_prototype_id: ManagementPrototypeId,
    pub func_id: FuncId,
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
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        // don't add binding if parent is locked
        SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;

        let func = Func::get_by_id(ctx, func_id).await?;
        ManagementPrototype::new(
            ctx,
            func.name.to_owned(),
            func.description.to_owned(),
            func.id,
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
            Some(prototype_id) => {
                let mut installable_schemas: HashSet<String> = CachedModule::latest_modules(ctx)
                    .await?
                    .into_iter()
                    .map(|m| m.schema_name)
                    .collect();

                let mut component_create_types = vec![];
                let mut component_input_types = vec![];
                for schema in Schema::list(ctx).await? {
                    // Since it's already installed, we don't include it in "installable_schemas"
                    installable_schemas.remove(schema.name());

                    let variant_id =
                        Schema::get_or_install_default_variant(ctx, schema.id()).await?;
                    let root_prop_id = SchemaVariant::get_root_prop_id(ctx, variant_id).await?;
                    let sv_type = Prop::ts_type(ctx, root_prop_id).await?;

                    let json_name = serde_json::to_string(schema.name())?;

                    let component_input_type = format!(
                        r#"
                            {{
                                kind: {json_name},
                                properties?: {sv_type},
                                geometry?: {{ [key: string]: Geometry }},
                                incomingConnections?: {{ [socket: string]: {INCOMING_CONNECTION_TYPE} | {INCOMING_CONNECTION_TYPE}[] }},
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
                                kind: {json_name},
                                properties?: {sv_type},
                                attributes?: {{ [path: string]: ValueOrSource }},
                                geometry?: Geometry | {{ [key: string]: Geometry }},
                                connect?: Connection[],
                                parent?: string,
                            }}
                        "#
                    );
                    component_create_types.push(component_create_type);
                    component_input_types.push(component_input_type);
                }

                // Add a generic type for any uninstalled modules, but with the names so that
                // you know what can be installed
                if !installable_schemas.is_empty() {
                    let kinds: Vec<String> = installable_schemas
                        .into_iter()
                        .sorted()
                        .map(|name| serde_json::to_string(&name))
                        .try_collect()?;
                    let kinds = kinds.join(" | ");
                    component_input_types.push(format!(
                        r#"
                            {{
                                kind: {kinds},
                                properties?: {{ [key: string]: unknown }},
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
                    ));
                    component_create_types.push(format!(
                        r#"
                            {{
                                kind: {kinds},
                                properties?: {{ [key: string]: any }},
                                attributes?: {{ [path: string]: ValueOrSource }},
                                geometry?: Geometry | {{ [key: string]: Geometry }},
                                connect?: Connection[],
                                parent?: string,
                            }}
                        "#
                    ))
                }

                let component_create_type = component_create_types.join("|\n");
                let component_input_type = component_input_types.join("|\n");

                let mut this_incoming_connections = "    {\n".to_string();
                let variant_id =
                    ManagementPrototype::get_schema_variant_id(ctx, prototype_id).await?;
                let this_root_prop_id = SchemaVariant::get_root_prop_id(ctx, variant_id).await?;
                let this_component_iface = Prop::ts_type(ctx, this_root_prop_id).await?;
                for input_socket in SchemaVariant::list_all_sockets(ctx, variant_id).await?.1 {
                    let json_input_socket_name = serde_json::to_string(input_socket.name())?;
                    let type_qualifier = match input_socket.arity() {
                        SocketArity::One => " | undefined",
                        SocketArity::Many => "[]",
                    };
                    this_incoming_connections.push_str(&format!(
                            "      {json_input_socket_name}: {INCOMING_CONNECTION_TYPE}{type_qualifier},\n"
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
        };

        Ok(format!(
            r#"
type ValueSource =
  | {{ component: string; path: string }}
  | {{ value?: JsonValue }}
  | null;
type JsonValue =
  | string
  | number
  | boolean
  | null
  | {{ string: JsonValue }}
  | JsonValue[];
type ValueOrSource = JsonValue | {{ $source: ValueSource }};
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
    create?: {{ [name: string]: {component_create_type} }},
    update?: {{ [name: string]: {{
        properties?: {{ [name: string]: unknown }},
        attributes?: {{ [path: string]: ValueOrSource }},
        geometry?: {{ [view: string]: Geometry }},
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
    geometry: {{ [view: string]: Geometry }},
    incomingConnections: {this_incoming_connections},
  }},
  components: {{ [name: string]: {component_input_type} }},
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
            let schema_variant_id =
                ManagementPrototype::get_schema_variant_id(ctx, management_prototype_id).await;
            match schema_variant_id {
                Ok(schema_variant_id) => {
                    bindings.push(FuncBinding::Management(ManagementBinding {
                        schema_variant_id,
                        func_id,
                        management_prototype_id,
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

        ManagementPrototype::remove(ctx, self.management_prototype_id).await?;

        Self::create_management_binding(ctx, new_func_id, schema_variant_id).await?;

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
}
