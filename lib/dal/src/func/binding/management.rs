use std::collections::{
    BTreeMap,
    HashSet,
};

use itertools::Itertools;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::SchemaId;
use telemetry::prelude::*;

use super::{
    EventualParent,
    FuncBinding,
    FuncBindingResult,
};
use crate::{
    DalContext,
    EdgeWeightKind,
    Func,
    FuncId,
    Prop,
    Schema,
    SchemaVariant,
    SchemaVariantId,
    cached_module::CachedModule,
    func::binding::FuncBindingError,
    management::prototype::{
        ManagementPrototype,
        ManagementPrototypeId,
        ManagementPrototypeParent,
    },
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ManagementBinding {
    pub schema_ids: Option<Vec<SchemaId>>,
    pub schema_variant_id: Option<SchemaVariantId>,
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
        schema_ids: Option<Vec<SchemaId>>,
        schema_variant_id: Option<SchemaVariantId>,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let func = Func::get_by_id(ctx, func_id).await?;

        let mut prototype = None;

        // don't add binding if schema_variant_id is locked
        if let Some(schema_variant_id) = schema_variant_id {
            SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;
            prototype = Some(
                ManagementPrototype::new(
                    ctx,
                    func.name.to_owned(),
                    func.description.to_owned(),
                    func.id,
                    ManagementPrototypeParent::SchemaVariant(schema_variant_id),
                )
                .await?,
            );
        }

        if let Some(schema_ids) = schema_ids {
            match &prototype {
                Some(prototype) => {
                    for schema_id in schema_ids {
                        Schema::add_edge_to_management_prototype(
                            ctx,
                            schema_id,
                            prototype.id(),
                            EdgeWeightKind::ManagementPrototype,
                        )
                        .await?;
                    }
                }
                None => {
                    prototype = Some(
                        ManagementPrototype::new(
                            ctx,
                            func.name.to_owned(),
                            func.description.to_owned(),
                            func.id,
                            ManagementPrototypeParent::Schemas(schema_ids),
                        )
                        .await?,
                    );
                }
            }

            // this should always be some here
            if let Some(prototype) = &prototype {
                ManagementPrototype::promote_to_overlay(ctx, prototype.id()).await?;
            }
        }

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

        let mut default_variant_root_props = BTreeMap::new();
        let mut ts_type_cache = BTreeMap::new();

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

                    let variant_id = Schema::default_variant_id(ctx, schema.id()).await?;
                    let root_prop_id = SchemaVariant::get_root_prop_id(ctx, variant_id).await?;
                    default_variant_root_props.insert(schema.id(), root_prop_id);
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

                    ts_type_cache.insert(root_prop_id, sv_type);

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

                let mut this_component_iface = String::new();
                let mut root_prop_ids = vec![];
                if let Some(schema_ids) = ManagementPrototype::schema_ids(ctx, prototype_id).await?
                {
                    for schema_id in schema_ids {
                        let root_prop_id = match default_variant_root_props.get(&schema_id) {
                            Some(root_prop_id) => *root_prop_id,
                            None => {
                                let variant_id = Schema::default_variant_id(ctx, schema_id).await?;
                                SchemaVariant::get_root_prop_id(ctx, variant_id).await?
                            }
                        };
                        root_prop_ids.push(root_prop_id);
                    }
                }

                if let Some(schema_variant_id) =
                    ManagementPrototype::schema_variant_id(ctx, prototype_id).await?
                {
                    let root_prop_id =
                        SchemaVariant::get_root_prop_id(ctx, schema_variant_id).await?;
                    root_prop_ids.push(root_prop_id);
                }

                for root_prop_id in root_prop_ids {
                    let ts_type = match ts_type_cache.get(&root_prop_id) {
                        Some(ts_type) => ts_type.clone(),
                        None => Prop::ts_type(ctx, root_prop_id).await?,
                    };

                    if !this_component_iface.is_empty() {
                        this_component_iface.push_str(" | ");
                    }
                    this_component_iface.push_str(&ts_type);
                }

                (
                    this_component_iface,
                    DEFAULT_THIS_INCOMING_CONNECTIONS.into(),
                    component_create_type,
                    component_input_type,
                )
            }
            None => default_types,
        };

        Ok(format!(
            r#"
type ValueSource =
  | {{ component: string; path: string; func?: string; }}
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
    sources: {{ [path: string]: {{ component: string; path: string; func?: string; }} }},
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
                match ManagementPrototype::schema_variant_id(ctx, management_prototype_id).await {
                    Ok(sv_id) => sv_id,
                    Err(err) => {
                        error!(error=?err, "Could not get bindings for func_id {func_id}");
                        continue;
                    }
                };

            let schema_ids =
                match ManagementPrototype::schema_ids(ctx, management_prototype_id).await {
                    Ok(s_ids) => s_ids,
                    Err(err) => {
                        error!(error=?err, "Could not get bindings for func_id {func_id}");
                        continue;
                    }
                };

            bindings.push(FuncBinding::Management(ManagementBinding {
                schema_ids,
                schema_variant_id,
                func_id,
                management_prototype_id,
            }));
        }

        Ok(bindings)
    }

    pub async fn port_binding_to_new_func(
        &self,
        ctx: &DalContext,
        new_func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        ManagementPrototype::remove(ctx, self.management_prototype_id).await?;

        Self::create_management_binding(
            ctx,
            new_func_id,
            self.schema_ids.clone(),
            self.schema_variant_id,
        )
        .await?;

        FuncBinding::for_func_id(ctx, new_func_id).await
    }

    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.action.delete_management_binding"
    )]
    pub async fn delete_management_binding(
        ctx: &DalContext,
        management_prototype_id: ManagementPrototypeId,
    ) -> FuncBindingResult<EventualParent> {
        let eventual_parent: EventualParent;

        // don't delete binding if parent is locked
        if let Some(schema_variant_id) =
            ManagementPrototype::schema_variant_id(ctx, management_prototype_id).await?
        {
            SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;
            eventual_parent = EventualParent::SchemaVariant(schema_variant_id);
        } else if let Some(schema_ids) =
            ManagementPrototype::schema_ids(ctx, management_prototype_id).await?
        {
            eventual_parent = EventualParent::Schemas(schema_ids);
        } else {
            return Err(FuncBindingError::ManagementPrototypeNoParent(
                management_prototype_id,
            ));
        }

        ManagementPrototype::remove(ctx, management_prototype_id).await?;

        Ok(eventual_parent)
    }
}
