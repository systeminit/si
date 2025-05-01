use std::collections::VecDeque;

use dal::{
    AttributeValue,
    Component,
    ComponentId,
    DalContext,
    Prop,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    SocketArity,
    component::socket::{
        ComponentInputSocket,
        ComponentOutputSocket,
    },
    diagram::{
        geometry::Geometry,
        view::View,
    },
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::Value;
use si_id::{
    AttributeValueId,
    PropId,
    ViewId,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};
use utoipa::ToSchema;

use crate::service::v1::ComponentsResult;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentViewV1 {
    #[schema(value_type = String)]
    pub id: ComponentId,
    #[schema(value_type = String)]
    pub schema_id: SchemaId,
    #[schema(value_type = String)]
    pub schema_variant_id: SchemaVariantId,
    pub sockets: Vec<SocketViewV1>,
    // this is everything below root/domain - the whole tree! (not including root/domain itself)
    pub domain_props: Vec<ComponentPropViewV1>,
    // from root/resource_value NOT root/resource/payload
    pub resource_props: Vec<ComponentPropViewV1>,
    // maps to root/si/name
    pub name: String,
    // maps to root/si/resource_id
    pub resource_id: String,
    pub to_delete: bool,
    pub can_be_upgraded: bool,
    // current connections to/from this component (should these be separated?)
    pub connections: Vec<ConnectionViewV1>,
    // what views this component is in
    pub views: Vec<ViewV1>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ComponentPropViewV1 {
    #[schema(value_type = String)]
    pub id: AttributeValueId, // I know prop view with an id for an AV...
    #[schema(value_type = String)]
    pub prop_id: PropId,
    #[schema(value_type = Object)]
    pub value: Option<Value>,
    #[schema(value_type = String, example = "path/to/prop")]
    pub path: String,
    // todo: Validation
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ViewV1 {
    #[schema(value_type = String)]
    pub id: ViewId,
    pub name: String,
    pub is_default: bool,
}

#[derive(AsRefStr, Clone, Debug, Deserialize, Display, Eq, PartialEq, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ConnectionViewV1 {
    Incoming(IncomingConnectionViewV1),
    Outgoing(OutgoingConnectionViewV1),
    Managing(ManagingConnectionViewV1),
    ManagedBy(ManagedByConnectionViewV1),
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct IncomingConnectionViewV1 {
    #[schema(value_type = String)]
    pub from_component_id: ComponentId,
    pub from_component_name: String,
    pub from: String, // from socket or prop
    pub to: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct OutgoingConnectionViewV1 {
    #[schema(value_type = String)]
    pub to_component_id: ComponentId,
    pub to_component_name: String,
    pub from: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManagingConnectionViewV1 {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
    pub component_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManagedByConnectionViewV1 {
    #[schema(value_type = String)]
    pub component_id: ComponentId,
    pub component_name: String,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
    ToSchema,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum SocketDirection {
    Input,
    Output,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SocketViewV1 {
    pub id: String,
    pub name: String,
    pub direction: SocketDirection,
    #[schema(value_type = String, example = "one", example = "many")]
    pub arity: SocketArity,
    #[schema(value_type = Object)]
    pub value: Option<serde_json::Value>,
}

impl ComponentViewV1 {
    pub async fn assemble(ctx: &DalContext, component_id: ComponentId) -> ComponentsResult<Self> {
        let component = Component::get_by_id(ctx, component_id).await?;
        let schema_variant = component.schema_variant(ctx).await?;
        // lets get all sockets
        let mut sockets = Vec::new();
        let (output_sockets, input_sockets) =
            SchemaVariant::list_all_sockets(ctx, schema_variant.id()).await?;
        for output in output_sockets {
            sockets.push(SocketViewV1 {
                id: output.id().to_string(),
                name: output.name().to_owned(),
                direction: SocketDirection::Output,
                arity: output.arity(),
                value: ComponentOutputSocket::value_for_output_socket_id_for_component_id_opt(
                    ctx,
                    component_id,
                    output.id(),
                )
                .await?,
            });
        }

        for input in input_sockets {
            // TODO(brit): figure out connection annotations
            sockets.push(SocketViewV1 {
                id: input.id().to_string(),
                name: input.name().to_owned(),
                direction: SocketDirection::Input,
                arity: input.arity(),
                value: ComponentInputSocket::value_for_input_socket_id_for_component_id_opt(
                    ctx,
                    component_id,
                    input.id(),
                )
                .await?,
            });
        }
        // Socket Connections
        let mut connections = Vec::new();
        let incoming = component.incoming_connections(ctx).await?;
        for input in incoming {
            connections.push(ConnectionViewV1::Incoming(IncomingConnectionViewV1 {
                from_component_id: input.from_component_id,
                from_component_name: Component::name_by_id(ctx, input.from_component_id).await?,
                from: input.from_output_socket_id.to_string(),
                to: input.to_input_socket_id.to_string(),
            }));
        }
        let outgoing = component.outgoing_connections(ctx).await?;
        for output in outgoing {
            connections.push(ConnectionViewV1::Outgoing(OutgoingConnectionViewV1 {
                to_component_id: output.to_component_id,
                to_component_name: Component::name_by_id(ctx, output.to_component_id).await?,
                from: output.from_output_socket_id.to_string(),
            }));
        }

        // Management Connections
        // Who is managing this component?
        let managers = Component::managers_by_id(ctx, component_id).await?;
        for manager in managers {
            connections.push(ConnectionViewV1::ManagedBy(ManagedByConnectionViewV1 {
                component_id: manager,
                component_name: Component::name_by_id(ctx, manager).await?,
            }));
        }
        // Who is this component managing?
        let managing = component.get_managed(ctx).await?;
        for managed in managing {
            connections.push(ConnectionViewV1::Managing(ManagingConnectionViewV1 {
                component_id: managed,
                component_name: Component::name_by_id(ctx, managed).await?,
            }));
        }

        // Domain Props
        let mut domain_props = Vec::new();
        let domain_root_av = component.domain_prop_attribute_value(ctx).await?;
        let mut work_queue = VecDeque::new();
        let domain_values = AttributeValue::get_child_av_ids_in_order(ctx, domain_root_av).await?;
        work_queue.extend(domain_values);
        while let Some(av) = work_queue.pop_front() {
            let attribute_value = AttributeValue::get_by_id(ctx, av).await?;
            let prop_id = AttributeValue::prop_id(ctx, av).await?;
            let is_hidden_prop = Prop::get_by_id(ctx, prop_id).await?.hidden;
            if !is_hidden_prop {
                let view = ComponentPropViewV1 {
                    id: av,
                    prop_id,
                    value: attribute_value.view(ctx).await?,
                    path: AttributeValue::get_path_for_id(ctx, av)
                        .await?
                        .unwrap_or_else(String::new),
                };
                domain_props.push(view);
                let children = AttributeValue::get_child_av_ids_in_order(ctx, av).await?;

                work_queue.extend(children);
            }
        }
        // sort alphabetically by path
        domain_props.sort_by_key(|view| view.path.to_lowercase());

        // Resource Props
        let mut resource_props = Vec::new();
        let resource_value_root_av = component.resource_value_prop_attribute_value(ctx).await?;
        let mut work_queue = VecDeque::new();
        let resource_value_values =
            AttributeValue::get_child_av_ids_in_order(ctx, resource_value_root_av).await?;
        work_queue.extend(resource_value_values);
        while let Some(av) = work_queue.pop_front() {
            let attribute_value = AttributeValue::get_by_id(ctx, av).await?;

            let view = ComponentPropViewV1 {
                id: av,
                prop_id: AttributeValue::prop_id(ctx, av).await?,
                value: attribute_value.view(ctx).await?,
                path: AttributeValue::get_path_for_id(ctx, av)
                    .await?
                    .unwrap_or_else(String::new),
            };
            resource_props.push(view);
            let children = AttributeValue::get_child_av_ids_in_order(ctx, av).await?;

            work_queue.extend(children);
        }

        // sort alphabetically by path
        resource_props.sort_by_key(|view| view.path.to_lowercase());

        // get views
        let mut views = Vec::new();
        let geos = Geometry::by_view_for_component_id(ctx, component_id).await?;
        for view_id in geos.keys() {
            let view = View::get_by_id(ctx, *view_id).await?;
            views.push(ViewV1 {
                id: *view_id,
                name: view.name().to_string(),
                is_default: view.is_default(ctx).await?,
            });
        }

        let result = ComponentViewV1 {
            id: component_id,
            schema_id: schema_variant.schema_id(ctx).await?,
            schema_variant_id: schema_variant.id(),
            sockets,
            domain_props,
            resource_props,
            name: component.name(ctx).await?,
            resource_id: component.resource_id(ctx).await?,
            to_delete: component.to_delete(),
            can_be_upgraded: component.can_be_upgraded(ctx).await?,
            connections,
            views,
        };
        Ok(result)
    }
}
