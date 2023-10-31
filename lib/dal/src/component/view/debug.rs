use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use thiserror::Error;

use crate::{
    func::execution::{FuncExecution, FuncExecutionError},
    socket::SocketEdgeKind,
    AttributePrototype, AttributeValue, AttributeValueId, AttributeValuePayload, Component,
    ComponentId, DalContext, ExternalProvider, ExternalProviderId, Func, FuncBinding,
    FuncBindingError, FuncBindingReturnValue, FuncBindingReturnValueError, InternalProvider,
    InternalProviderId, Prop, PropId, PropKind, SchemaVariantId, SecretError, SecretId, Socket,
    SocketId, StandardModel, StandardModelError,
};

type ComponentDebugViewResult<T> = Result<T, ComponentDebugViewError>;

/// A generated view for an [`Component`](crate::Component) that contains metadata about each of
/// the components attributes. Used for constructing a debug view of the component and also for
/// cloning a component
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDebugView {
    pub name: String,
    pub schema_variant_id: SchemaVariantId,
    pub attributes: Vec<AttributeDebugView>,
    pub input_sockets: Vec<AttributeDebugView>,
    pub output_sockets: Vec<AttributeDebugView>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeDebugView {
    pub path: String,
    pub parent_info: Option<ParentInfo>,
    pub attribute_value: AttributeValue,
    pub func: Func,
    pub func_binding: FuncBinding,
    pub func_binding_return_value: FuncBindingReturnValue,
    pub func_execution: FuncExecution,
    pub prop: Option<Prop>,
    pub internal_provider: Option<InternalProvider>,
    pub external_provider: Option<ExternalProvider>,
    pub prototype: AttributePrototype,
    pub array_index: Option<i64>,
    pub implicit_attribute_value: Option<AttributeValue>,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ComponentDebugViewError {
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error("Attribute Value tree badly constructed with root prop of {0}")]
    AttributeValueTreeBad(AttributeValueId),
    #[error("component error: {0}")]
    Component(String),
    #[error("external provider not found for output socket: {0}")]
    ExternalProviderNotFoundForInputSocket(SocketId),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    FuncExecution(#[from] FuncExecutionError),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error("internal provider not found for input socket: {0}")]
    InternalProviderNotFoundForInputSocket(SocketId),
    #[error("json pointer not found: {1:?} at {0}")]
    JSONPointerNotFound(serde_json::Value, String),
    #[error("no attribute value found for context {0:?}")]
    NoAttributeValue(AttributeReadContext),
    #[error("no internal provider for prop {0}")]
    NoInternalProvider(PropId),
    #[error("no root prop found for schema variant {0}")]
    NoRootProp(SchemaVariantId),
    #[error("schema variant not found for component {0}")]
    NoSchemaVariant(ComponentId),
    #[error("component not found {0}")]
    NotFound(ComponentId),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    Secret(#[from] SecretError),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Socket(#[from] SocketError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    UlidDecode(#[from] ulid::DecodeError),
}

pub enum AttributeDebugInput<'a> {
    ComponentSocket((Socket, ComponentId)),
    AttributeValuePayload {
        payload: &'a AttributeValuePayload,
        implicit_attribute_value: Option<AttributeValue>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParentInfo {
    pub value: AttributeValue,
    pub kind: PropKind,
    pub path: String,
    pub key: Option<String>,
    pub array_index: Option<i64>,
}

impl ComponentDebugView {
    pub async fn new(ctx: &DalContext, component: &Component) -> ComponentDebugViewResult<Self> {
        let debug_view_start = Instant::now();

        let schema_variant = component
            .schema_variant(ctx)
            .await
            .map_err(|e| ComponentDebugViewError::Component(e.to_string()))?
            .ok_or(ComponentError::NoSchemaVariant(*component.id()))
            .map_err(|e| ComponentDebugViewError::Component(e.to_string()))?;

        let root_prop_id = schema_variant
            .root_prop_id()
            .ok_or(ComponentDebugViewError::NoRootProp(*schema_variant.id()))?;

        let root_av_context = AttributeReadContext {
            prop_id: Some(*root_prop_id),
            internal_provider_id: Some(InternalProviderId::NONE),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(*component.id()),
        };

        let root_prop_av = AttributeValue::find_for_context(ctx, root_av_context)
            .await?
            .ok_or(AttributeValueError::NotFoundForReadContext(root_av_context))?;

        let view_context = AttributeReadContext {
            prop_id: None,
            internal_provider_id: Some(InternalProviderId::NONE),
            external_provider_id: Some(ExternalProviderId::NONE),
            component_id: Some(*component.id()),
        };

        let mut initial_work = AttributeValue::list_payload_for_read_context_and_root(
            ctx,
            *root_prop_av.id(),
            view_context,
        )
        .await?;

        // We sort the work queue according to the order of every nested IndexMap. This ensures that
        // when we reconstruct the final shape, we don't have to worry about the order that things
        // appear in.
        let attribute_value_order: Vec<AttributeValueId> = initial_work
            .iter()
            .filter_map(|avp| avp.attribute_value.index_map())
            .flat_map(|index_map| index_map.order())
            .copied()
            .collect();
        initial_work.sort_by_cached_key(|avp| {
            attribute_value_order
                .iter()
                .position(|attribute_value_id| attribute_value_id == avp.attribute_value.id())
                .unwrap_or(0)
        });

        let mut index_map: HashMap<PropId, i64> = HashMap::new();
        let mut work_queue = VecDeque::from(initial_work);
        let mut parent_queue: VecDeque<Option<ParentInfo>> = VecDeque::from([None]);
        let mut attributes = vec![];

        while !work_queue.is_empty() {
            let mut unprocessed = vec![];

            if let Some(parent_info) = parent_queue.pop_front() {
                let mut current_parent = parent_info;

                while let Some(payload) = work_queue.pop_front() {
                    if current_parent.as_ref().map(|parent| *parent.value.id())
                        == payload.parent_attribute_value_id
                    {
                        let current_prop_name = payload.prop.name();
                        let current_parent_path = current_parent
                            .as_ref()
                            .map(|p| p.path.as_str())
                            .unwrap_or("");

                        let prop_full_path = match current_parent.as_ref().map(|p| p.kind) {
                            Some(PropKind::Array) => {
                                let array_index = *index_map.get(payload.prop.id()).unwrap_or(&0);
                                let path = format!(
                                    "{}/{}/{}",
                                    current_parent_path, current_prop_name, array_index
                                );
                                index_map.insert(*payload.prop.id(), array_index + 1);
                                path
                            }
                            Some(PropKind::Map) => {
                                if let Some(key) = payload.attribute_value.key() {
                                    format!("{}/{}/{}", current_parent_path, current_prop_name, key)
                                } else {
                                    // This should be an error
                                    format!("{}/{}", current_parent_path, current_prop_name)
                                }
                            }
                            _ => format!("{}/{}", current_parent_path, current_prop_name),
                        };

                        let current_index = index_map.get(payload.prop.id()).map(|index| index - 1);

                        let implicit_attribute_value = if let Some(internal_provider) =
                            InternalProvider::find_for_prop(ctx, *payload.prop.id()).await?
                        {
                            let implicit_attribute_value_context = AttributeReadContext {
                                internal_provider_id: Some(*internal_provider.id()),
                                component_id: Some(*component.id()),
                                ..Default::default()
                            };
                            let implicit_attribute_value = AttributeValue::find_for_context(
                                ctx,
                                implicit_attribute_value_context,
                            )
                            .await?;

                            if let Some(implicit_attribute_value) = implicit_attribute_value {
                                if implicit_attribute_value.context.component_id().is_none() {
                                    None
                                } else {
                                    Some(implicit_attribute_value)
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        };

                        attributes.push(
                            Self::get_attribute_debug_view(
                                ctx,
                                AttributeDebugInput::AttributeValuePayload {
                                    payload: &payload,
                                    implicit_attribute_value,
                                },
                                current_parent.to_owned(),
                                Some(prop_full_path.to_owned()),
                                current_index,
                            )
                            .await?,
                        );

                        match payload.prop.kind() {
                            PropKind::Object | PropKind::Array | PropKind::Map => {
                                // The current parent is pushed back onto the queue and swapped
                                // with this value, which transforms this into a depth-first search
                                // (but preserves index-map ordering above)
                                parent_queue.push_front(current_parent);
                                // too much cloning!
                                //
                                let (key, array_index) = match &current_index {
                                    Some(index) => (None, Some(*index)),
                                    None => (payload.attribute_value.key.to_owned(), None),
                                };

                                current_parent = Some(ParentInfo {
                                    value: payload.attribute_value.to_owned(),
                                    kind: *payload.prop.kind(),
                                    path: prop_full_path,
                                    key,
                                    array_index,
                                });

                                // Since we've changed parents we need to reprocess the unprocessed
                                // in case they are children of this parent but were skipped
                                for unprocessed_payload in unprocessed {
                                    work_queue.push_front(unprocessed_payload);
                                }
                                unprocessed = vec![];

                                continue;
                            }
                            _ => {}
                        }
                    } else {
                        unprocessed.push(payload);
                    }
                }
                work_queue = VecDeque::from(unprocessed);

                // If we are out of roots but we have work left to process, something went wrong
                // with the attribute value structure in the database (we had an orphaned child
                // with no parent)
                if parent_queue.is_empty() && !work_queue.is_empty() {
                    return Err(ComponentDebugViewError::AttributeValueTreeBad(
                        *root_prop_av.id(),
                    ));
                }
            }
        }

        let attributes_duration = debug_view_start.elapsed();
        let sockets_start = Instant::now();

        let mut input_sockets = vec![];
        let mut output_sockets = vec![];

        for socket in Socket::list_for_component(ctx, *component.id()).await? {
            let socket_debug_data = Self::get_attribute_debug_view(
                ctx,
                AttributeDebugInput::ComponentSocket((socket, *component.id())),
                None,
                None,
                None,
            )
            .await?;

            if socket_debug_data.internal_provider.is_some() {
                input_sockets.push(socket_debug_data);
            } else {
                output_sockets.push(socket_debug_data)
            }
        }

        let sockets_duration = sockets_start.elapsed();

        dbg!(attributes_duration, sockets_duration);

        let name = component
            .name(ctx)
            .await
            .map_err(|e| ComponentDebugViewError::Component(format!("get name error: {}", e)))?;

        let debug_view = ComponentDebugView {
            name,
            schema_variant_id: *schema_variant.id(),
            attributes,
            input_sockets,
            output_sockets,
        };

        Ok(debug_view)
    }

    pub async fn get_attribute_debug_view(
        ctx: &DalContext,
        payload: AttributeDebugInput<'_>,
        parent_info: Option<ParentInfo>,
        path: Option<String>,
        array_index: Option<i64>,
    ) -> ComponentDebugViewResult<AttributeDebugView> {
        let (
            attribute_value,
            prop,
            internal_provider,
            external_provider,
            func_binding_return_value,
            path,
            implicit_attribute_value,
        ) = match payload {
            AttributeDebugInput::AttributeValuePayload {
                payload,
                implicit_attribute_value,
            } => {
                let func_binding_return_value = match &payload.func_binding_return_value {
                    Some(fbrv) => fbrv.to_owned(),
                    None => FuncBindingReturnValue::get_by_id(
                        ctx,
                        &payload.attribute_value.func_binding_return_value_id(),
                    )
                    .await?
                    .ok_or(FuncBindingReturnValueError::NotFound(
                        payload.attribute_value.func_binding_return_value_id(),
                    ))?,
                };

                let path = path.unwrap_or(payload.prop.name().into());

                (
                    payload.attribute_value.to_owned(),
                    Some(payload.prop.to_owned()),
                    None,
                    None,
                    func_binding_return_value,
                    path,
                    implicit_attribute_value,
                )
            }
            AttributeDebugInput::ComponentSocket((socket, component_id)) => {
                let (attribute_value, internal_provider, external_provider, path) =
                    match socket.edge_kind() {
                        SocketEdgeKind::ConfigurationInput => {
                            let internal_provider = socket.internal_provider(ctx).await?.ok_or(
                                ComponentDebugViewError::InternalProviderNotFoundForInputSocket(
                                    *socket.id(),
                                ),
                            )?;

                            let input_socket_value_context = AttributeReadContext {
                                prop_id: Some(PropId::NONE),
                                internal_provider_id: Some(*internal_provider.id()),
                                external_provider_id: Some(ExternalProviderId::NONE),
                                component_id: Some(component_id),
                            };
                            let attribute_value =
                                AttributeValue::find_for_context(ctx, input_socket_value_context)
                                    .await?
                                    .ok_or(AttributeValueError::NotFoundForReadContext(
                                        input_socket_value_context,
                                    ))?;

                            let name = internal_provider.name().to_owned();
                            (attribute_value, Some(internal_provider), None, name)
                        }
                        SocketEdgeKind::ConfigurationOutput => {
                            let external_provider = socket.external_provider(ctx).await?.ok_or(
                                ComponentDebugViewError::ExternalProviderNotFoundForInputSocket(
                                    *socket.id(),
                                ),
                            )?;

                            let input_socket_value_context = AttributeReadContext {
                                prop_id: Some(PropId::NONE),
                                internal_provider_id: Some(InternalProviderId::NONE),
                                external_provider_id: Some(*external_provider.id()),
                                component_id: Some(component_id),
                            };
                            let attribute_value =
                                AttributeValue::find_for_context(ctx, input_socket_value_context)
                                    .await?
                                    .ok_or(AttributeValueError::NotFoundForReadContext(
                                        input_socket_value_context,
                                    ))?;

                            let name = external_provider.name().to_owned();
                            (attribute_value, None, Some(external_provider), name)
                        }
                    };

                let func_binding_return_value = FuncBindingReturnValue::get_by_id(
                    ctx,
                    &attribute_value.func_binding_return_value_id(),
                )
                .await?
                .ok_or(FuncBindingReturnValueError::NotFound(
                    attribute_value.func_binding_return_value_id(),
                ))?;

                (
                    attribute_value,
                    None,
                    internal_provider,
                    external_provider,
                    func_binding_return_value,
                    path,
                    None,
                )
            }
        };

        let prototype = attribute_value.attribute_prototype(ctx).await?.ok_or(
            AttributeValueError::AttributePrototypeNotFound(
                *attribute_value.id(),
                *ctx.visibility(),
            ),
        )?;

        let func = Func::get_by_id(ctx, &prototype.func_id())
            .await?
            .ok_or(FuncError::NotFound(prototype.func_id()))?;

        let func_binding = FuncBinding::get_by_id(ctx, &attribute_value.func_binding_id())
            .await?
            .ok_or(FuncBindingError::NotFound(
                attribute_value.func_binding_id(),
            ))?;

        let func_execution =
            FuncExecution::get_by_pk(ctx, &func_binding_return_value.func_execution_pk()).await?;

        Ok(AttributeDebugView {
            path,
            parent_info,
            attribute_value,
            func,
            func_binding,
            func_binding_return_value,
            func_execution,
            prop,
            internal_provider,
            external_provider,
            prototype,
            array_index,
            implicit_attribute_value,
        })
    }
}
