use std::collections::{HashMap, VecDeque};

use axum::extract::Query;
use axum::Json;

use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::{
    socket::SocketEdgeKind, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    AttributeValueId, AttributeView, Component, ComponentId, DalContext, ExternalProviderId, Func,
    FuncArgument, FuncBinding, FuncId, InternalProvider, InternalProviderId, Prop, PropId, Socket,
    StandardModel, Visibility,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDebugView {
    attributes: Vec<AttributeDebugView>,
    input_sockets: Vec<AttributeDebugView>,
    output_sockets: Vec<AttributeDebugView>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeDebugView {
    name: String,
    path: String,
    debug_data: AttributeMetadataView,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeMetadataView {
    pub value_id: AttributeValueId,
    pub proxy_for: Option<AttributeValueId>,
    pub func_name: String,
    pub func_id: FuncId,
    pub func_args: serde_json::Value,
    pub arg_sources: HashMap<String, Option<String>>,
    pub visibility: Visibility,
    pub value: Option<serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DebugComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

type DebugComponentResponse = ComponentDebugView;

pub async fn debug_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<DebugComponentRequest>,
) -> ComponentResult<Json<DebugComponentResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentDebug("component not found".into()))?;

    let schema_variant =
        component
            .schema_variant(&ctx)
            .await?
            .ok_or(ComponentError::ComponentDebug(
                "schema variant not found for component".into(),
            ))?;

    let root_prop_id = schema_variant
        .root_prop_id()
        .ok_or(ComponentError::ComponentDebug(
            "could not get root prop for schema variant".into(),
        ))?;

    let av_context = AttributeReadContext {
        prop_id: Some(*root_prop_id),
        internal_provider_id: Some(InternalProviderId::NONE),
        external_provider_id: Some(ExternalProviderId::NONE),
        component_id: Some(*component.id()),
    };

    let root_prop_av = AttributeValue::find_for_context(&ctx, av_context)
        .await?
        .ok_or(ComponentError::ComponentDebug(
            "could not get attribute value for root prop".into(),
        ))?;

    let view_context = AttributeReadContext {
        prop_id: None,
        internal_provider_id: Some(InternalProviderId::NONE),
        external_provider_id: Some(ExternalProviderId::NONE),
        component_id: Some(*component.id()),
    };

    let root_prop_view = AttributeView::new(&ctx, view_context, Some(*root_prop_av.id())).await?;

    let pointer_to_av_ids: HashMap<String, AttributeValueId> = root_prop_view
        .json_pointers_for_attribute_value_id()
        .iter()
        .map(|(k, v)| (v.to_owned(), k.to_owned()))
        .collect();

    let mut value_stack = VecDeque::from([(
        "/root".to_string(),
        "root".to_string(),
        root_prop_view.value().to_owned(),
    )]);

    let mut attribute_views = vec![];
    let mut input_socket_views = vec![];
    let mut output_socket_views = vec![];

    while let Some((pointer, name, value)) = value_stack.pop_front() {
        if let Some(av_id) = pointer_to_av_ids.get(&pointer) {
            attribute_views.push(AttributeDebugView {
                path: pointer.to_owned(),
                name: name.to_owned(),
                debug_data: get_attribute_metadata(&ctx, *av_id).await?,
            });
        }

        match value {
            serde_json::Value::Object(map_object) => {
                for (k, v) in map_object.iter() {
                    let new_pointer = format!("{}/{}", &pointer, &k);
                    value_stack.push_front((new_pointer, k.to_owned(), v.to_owned()));
                }
            }
            serde_json::Value::Array(array) => {
                for (pos, v) in array.iter().enumerate() {
                    let position_string = format!("{}", &pos);
                    let new_pointer = format!("{}/{}", &pointer, &position_string);

                    value_stack.push_front((new_pointer, position_string, v.to_owned()));
                }
            }
            _ => {}
        }
    }

    let mut input_sockets = vec![];
    let mut output_sockets = vec![];

    for socket in Socket::list_for_component(&ctx, request.component_id)
        .await
        .map_err(|e| ComponentError::ComponentDebug(e.to_string()))?
    {
        match socket.edge_kind() {
            SocketEdgeKind::ConfigurationInput => {
                let internal_provider = socket
                    .internal_provider(&ctx)
                    .await
                    .map_err(|e| ComponentError::ComponentDebug(e.to_string()))?
                    .ok_or(ComponentError::ComponentDebug(
                        "could not find internal provider for an input socket".into(),
                    ))?;

                let input_socket_value_context = AttributeReadContext {
                    prop_id: Some(PropId::NONE),
                    internal_provider_id: Some(*internal_provider.id()),
                    external_provider_id: Some(ExternalProviderId::NONE),
                    component_id: Some(*component.id()),
                };
                let socket_value =
                    AttributeValue::find_for_context(&ctx, input_socket_value_context)
                        .await?
                        .ok_or(ComponentError::ComponentDebug(
                            "could not find attribute value for an input socket".into(),
                        ))?;

                input_sockets.push((
                    internal_provider.name().to_string(),
                    get_attribute_metadata(&ctx, *socket_value.id()).await?,
                ))
            }
            SocketEdgeKind::ConfigurationOutput => {
                let external_provider = socket
                    .external_provider(&ctx)
                    .await
                    .map_err(|e| ComponentError::ComponentDebug(e.to_string()))?
                    .ok_or(ComponentError::ComponentDebug(
                        "could not find external_provider for an output socket".into(),
                    ))?;

                let output_socket_value_context = AttributeReadContext {
                    prop_id: Some(PropId::NONE),
                    internal_provider_id: Some(InternalProviderId::NONE),
                    external_provider_id: Some(*external_provider.id()),
                    component_id: Some(*component.id()),
                };
                let socket_value =
                    AttributeValue::find_for_context(&ctx, output_socket_value_context)
                        .await?
                        .ok_or(ComponentError::ComponentDebug(
                            "could not find attribute value for an output socket".into(),
                        ))?;

                output_sockets.push((
                    external_provider.name().to_string(),
                    get_attribute_metadata(&ctx, *socket_value.id()).await?,
                ))
            }
        }
    }

    for (name, metadata) in input_sockets {
        input_socket_views.push(AttributeDebugView {
            path: "Input Socket".to_owned(),
            name,
            debug_data: metadata,
        });
    }

    for (name, metadata) in output_sockets {
        output_socket_views.push(AttributeDebugView {
            path: "Out Socket".to_owned(),
            name,
            debug_data: metadata,
        });
    }

    let component_view = ComponentDebugView {
        attributes: attribute_views,
        input_sockets: input_socket_views,
        output_sockets: output_socket_views,
    };

    Ok(Json(component_view))
}

async fn get_attribute_metadata(
    ctx: &DalContext,
    value_id: AttributeValueId,
) -> ComponentResult<AttributeMetadataView> {
    let value =
        AttributeValue::get_by_id(ctx, &value_id)
            .await?
            .ok_or(ComponentError::ComponentDebug(
                "could not find attribute value".into(),
            ))?;
    let mut arg_sources = HashMap::new();

    let prototype = value
        .attribute_prototype(ctx)
        .await?
        .ok_or(ComponentError::ComponentDebug(
            "could not find attribute prototype for value".into(),
        ))?;

    let proxy_for = value.proxy_for_attribute_value_id().copied();

    let func =
        Func::get_by_id(ctx, &prototype.func_id())
            .await?
            .ok_or(ComponentError::ComponentDebug(
                "could not find func used to set value".into(),
            ))?;

    let fb = FuncBinding::get_by_id(ctx, &value.func_binding_id())
        .await?
        .ok_or(ComponentError::ComponentDebug(
            "could not find func binding used to set value".into(),
        ))?;
    let func_args = fb.args().to_owned();

    for apa in
        AttributePrototypeArgument::list_for_attribute_prototype(ctx, *prototype.id()).await?
    {
        let arg = FuncArgument::get_by_id(ctx, &apa.func_argument_id())
            .await?
            .ok_or(ComponentError::ComponentDebug(
                "could not find func argument".into(),
            ))?;

        let internal_provider_id = apa.internal_provider_id();
        let input_ip_name = if internal_provider_id.is_some() {
            let ip = InternalProvider::get_by_id(ctx, &internal_provider_id)
                .await?
                .ok_or(ComponentError::ComponentDebug(
                    "could not find internal provider for input".into(),
                ))?;

            let prop_id = *ip.prop_id();
            let path = if prop_id == PropId::NONE {
                format!("Input Socket: {}", ip.name())
            } else {
                let prop =
                    Prop::get_by_id(ctx, &prop_id)
                        .await?
                        .ok_or(ComponentError::ComponentDebug(
                            "could not find prop for provider for input".into(),
                        ))?;

                format!("Prop: /{}", prop.path().with_replaced_sep("/"))
            };

            Some(path)
        } else {
            None
        };

        arg_sources.insert(arg.name().into(), input_ip_name);
    }

    Ok(AttributeMetadataView {
        value_id,
        proxy_for,
        func_name: func.name().into(),
        func_id: func.id().to_owned(),
        func_args,
        arg_sources,
        visibility: value.visibility().to_owned(),
        value: value.get_unprocessed_value(ctx).await?,
    })
}
