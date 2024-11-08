use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{
    management::prototype::{ManagementPrototype, ManagementPrototypeId},
    DalContext, Func, FuncId, Prop, Schema, SchemaId, SchemaVariant, SchemaVariantId,
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

        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        ManagementPrototype::new(
            ctx,
            func.name.to_owned(),
            func.description.to_owned(),
            func.id,
            None,
            schema_variant_id,
        )
        .await?;

        FuncBinding::for_func_id(ctx, func_id).await
    }

    pub async fn compile_management_types(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<String> {
        let default_this_component = "object".to_string();
        let default_component_types = "object".to_string();
        let default_types = (default_this_component, default_component_types.to_owned());

        let (this_component_iface, component_types) =
            match ManagementPrototype::prototype_id_for_func_id(ctx, func_id).await? {
                Some(prototype_id) => {
                    match ManagementPrototype::get_by_id(ctx, prototype_id).await? {
                        Some(prototype) => {
                            let variant_id =
                                ManagementPrototype::get_schema_variant_id(ctx, prototype_id)
                                    .await?;
                            let root_prop = Prop::get_by_id(
                                ctx,
                                SchemaVariant::get_root_prop_id(ctx, variant_id).await?,
                            )
                            .await?;

                            let (_, reverse_map) = prototype.managed_schemas_map(ctx).await?;

                            let mut component_types = vec![];
                            for (schema_id, name) in reverse_map {
                                let variant_id =
                                    Schema::get_or_install_default_variant(ctx, schema_id).await?;

                                let root_prop = Prop::get_by_id(
                                    ctx,
                                    SchemaVariant::get_root_prop_id(ctx, variant_id).await?,
                                )
                                .await?;

                                let sv_type = root_prop.ts_type(ctx).await?;

                                let component_type = format!(
                                    r#"
                                {{
                                    kind: "{name}",
                                    properties?: {sv_type},
                                    geometry?: Geometry,
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
                                component_types.push(component_type);
                            }

                            let component_types = component_types.join("|\n");

                            (root_prop.ts_type(ctx).await?, component_types)
                        }
                        None => default_types,
                    }
                }
                None => default_types,
            };

        Ok(format!(
            r#"
type Geometry = {{
    x: number,
    y: number,
    width?: number,
    height?: number,
}};
type Output = {{
  status: 'ok' | 'error';
  ops?: {{
    create?: {{ [key: string]: {component_types} }},
    update?: {{ [key: string]: {{ 
        properties?: {{ [key: string]: unknown }}, 
        geometry?: Geometry, 
        connect?: {{
            add?: {{ from: string, to: {{ component: string; socket: string; }} }}[],
            remove?: {{ from: string, to: {{ component: string; socket: string; }} }}[],
        }},
        parent?: string,
    }} }},
    actions?: {{ [key: string]: {{
      add?: ("create" | "update" | "refresh" | "delete" | string)[];
      remove?: ("create" | "update" | "refresh" | "delete" | string)[];
    }} }}
  }},
  message?: string | null;
}};
type Input = {{
    thisComponent: {{
        properties: {this_component_iface},
        geometry: Geometry,
    }},
    components: {{ [key: string]: {component_types} }}
}};"#
        ))
    }

    pub(crate) async fn assemble_management_bindings(
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
