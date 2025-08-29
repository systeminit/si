use std::collections::{
    HashMap,
    HashSet,
};

use itertools::Itertools;
use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    AttributePrototypeArgumentId,
    ComponentId,
    FuncId,
    InputSocketId,
    OutputSocketId,
    PropId,
};

use super::{
    WorkspaceSnapshotGraphError,
    func_produces_array,
    is_identity_func,
    is_normalize_to_array_func,
    prop_path_from_root,
};
use crate::{
    Component,
    DalContext,
    Func,
    InputSocket,
    OutputSocket,
    Prop,
    PropKind,
    SchemaVariant,
    WsEvent,
    WsEventResult,
    WsPayload,
    func::intrinsics::IntrinsicFunc,
    workspace_snapshot::{
        content_address::ContentAddress,
        edge_weight::{
            EdgeWeight,
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        graph::{
            WorkspaceSnapshotGraphVCurrent,
            validator::WithGraph,
        },
        node_weight::{
            ArgumentTargets,
            NodeWeight,
            NodeWeightError,
            traits::SiNodeWeight as _,
        },
    },
};

pub fn connection_migrations_with_text(
    graph: &impl std::ops::Deref<Target = WorkspaceSnapshotGraphVCurrent>,
    inferred_connections: impl IntoIterator<Item = SocketConnection>,
) -> Vec<(ConnectionMigration, String)> {
    connection_migrations(graph, inferred_connections)
        .into_iter()
        .map(|migration| {
            let text = format!("{}", WithGraph(graph.deref(), &migration));
            (migration, text)
        })
        .collect()
}

pub fn connection_migrations(
    graph: &impl std::ops::Deref<Target = WorkspaceSnapshotGraphVCurrent>,
    inferred_connections: impl IntoIterator<Item = SocketConnection>,
) -> Vec<ConnectionMigration> {
    let inferred_connection_migrations =
        inferred_connections.into_iter().map(|socket_connection| {
            match PropConnection::equivalent_to_socket_connection(graph, &socket_connection) {
                Ok(prop_connections) => ConnectionMigration {
                    explicit_connection_id: None,
                    socket_connection: Some(socket_connection),
                    prop_connections,
                    issue: None,
                },
                Err(issue) => ConnectionMigration {
                    explicit_connection_id: None,
                    socket_connection: Some(socket_connection),
                    prop_connections: vec![],
                    issue: Some(issue),
                },
            }
        });
    let explicit_connection_migrations = graph
        .edges()
        .filter_map(|edge| SocketConnection::socket_connection_edge_opt(graph, edge))
        .map(|(apa_id, source_socket_id)| {
            ConnectionMigration::from_socket_connection_edge(graph, apa_id, source_socket_id)
        });
    let mut migrations = explicit_connection_migrations
        .chain(inferred_connection_migrations)
        .collect_vec();

    // Look for multiple connections to the same destination socket
    let mut seen_destination_props = HashMap::new();
    let mut dup_destination_props = HashSet::new();
    for migration in &migrations {
        for prop_connection in &migration.prop_connections {
            // If the path is an "append" path, we want to make sure we're not conflicting
            // with the parent
            let (component_id, ref path) = prop_connection.to;
            let (base_path, is_append) = match path.split_back() {
                Some((parent_path, last)) if last.is_next() => (parent_path, true),
                _ => (path.as_ref(), false),
            };
            // If we had a previous connection to this socket, mark it as a duplicate
            if let Some(old_is_append) =
                seen_destination_props.insert((component_id, base_path.to_buf()), is_append)
            {
                // If all of them are appends, it's not a problem.
                if !(old_is_append && is_append) {
                    dup_destination_props.insert((component_id, path.clone()));
                    dup_destination_props.insert((component_id, base_path.to_buf()));
                }
            }
        }
    }

    // If we have multiple connections to the same destination prop, we can't migrate any of them
    for migration in &mut migrations {
        if migration.prop_connections.iter().any(|prop_connection| {
            // If the path is an "append" path, we want to make sure we're not conflicting
            // with the parent
            let (component_id, ref path) = prop_connection.to;
            let (base_path, _) = match path.split_back() {
                Some((parent_path, last)) if last.is_next() => (parent_path.to_buf(), true),
                _ => (path.clone(), false),
            };
            dup_destination_props.contains(&(component_id, base_path))
        }) {
            // We dedup whether or not socket connections are migrateable; if there are 10
            // un-migrateable connections to the same prop, and one migrateable one, we
            // still can't migrate any of them because it would still be inaccurate!
            migration.issue = Some(ConnectionUnmigrateableBecause::MultipleConnectionsToSameProp);
        }
    }

    migrations
}

/// A migration from a socket connection to a prop connection (with possible status).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMigration {
    /// The APA for this connection (if it is explicit and not inferred)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit_connection_id: Option<AttributePrototypeArgumentId>,
    /// The socket connection data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_connection: Option<SocketConnection>,
    /// The prop connections we will migrate to
    pub prop_connections: Vec<PropConnection>,
    /// The reason we can't migrate this connection, if any
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issue: Option<ConnectionUnmigrateableBecause>,
}

impl ConnectionMigration {
    fn from_socket_connection_edge(
        graph: &WorkspaceSnapshotGraphVCurrent,
        apa_id: AttributePrototypeArgumentId,
        source_socket_id: OutputSocketId,
    ) -> Self {
        // Get the migration data
        let mut migration = ConnectionMigration {
            explicit_connection_id: Some(apa_id),
            socket_connection: None,
            prop_connections: vec![],
            issue: None,
        };
        match SocketConnection::from_socket_connection_edge(graph, apa_id, source_socket_id) {
            Ok(socket_connection) => {
                match PropConnection::equivalent_to_socket_connection(graph, &socket_connection) {
                    Ok(prop_connections) => migration.prop_connections = prop_connections,
                    Err(issue) => migration.issue = Some(issue),
                }
                migration.socket_connection = Some(socket_connection);
            }
            Err(issue) => migration.issue = Some(issue),
        }
        migration
    }

    pub async fn fmt_title(&self, ctx: &DalContext) -> String {
        let mut message = String::new();
        if self.fmt_title_fallible(ctx, &mut message).await.is_err() {
            message.push_str(" <UNFINISHED: WRITE ERROR>");
        }
        message
    }

    async fn fmt_title_fallible(
        &self,
        ctx: &DalContext,
        f: &mut impl std::fmt::Write,
    ) -> std::fmt::Result {
        if let Some(ref issue) = self.issue {
            write!(f, "ERROR ")?;
            issue.fmt_title(ctx, f).await?;
            write!(f, " | ")?;
        }
        if let Some(ref socket_connection) = self.socket_connection {
            socket_connection.fmt_title(ctx, f).await?;
            write!(f, " | ")?;
            for prop_connection in &self.prop_connections {
                write!(f, "to prop ")?;
                prop_connection.fmt_title(ctx, f).await?;
                write!(f, " | ")?;
            }
        }

        match self.explicit_connection_id {
            Some(apa_id) => write!(f, "(explicit connection APA {apa_id})")?,
            None => write!(f, "(inferred connection)")?,
        }

        Ok(())
    }

    pub fn to_string(
        &self,
        graph: impl std::ops::Deref<Target = WorkspaceSnapshotGraphVCurrent>,
    ) -> String {
        format!("{}", WithGraph(graph.deref(), self))
    }
}

/// Data for a socket connection.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct SocketConnection {
    pub from: (ComponentId, OutputSocketId),
    pub to: (ComponentId, InputSocketId),
}

impl SocketConnection {
    /// Get source and destination data from an explicit socket connection edge in the graph
    ///
    ///   AttributePrototypeArgumentId --|PrototypeArgumentValue|-> OutputSocketId
    ///
    pub fn from_socket_connection_edge(
        graph: &WorkspaceSnapshotGraphVCurrent,
        apa_id: AttributePrototypeArgumentId,
        source_socket_id: OutputSocketId,
    ) -> Result<Self, ConnectionUnmigrateableBecause> {
        let apa = graph.get_node_index_by_id(apa_id)?;

        // Get the destination input socket
        //
        //     Dest Socket --|Prototype|-> . --|PrototypeArgument|-> Connection APA
        //
        let destination_socket_id = {
            let prototype = graph.source(apa, EdgeWeightKind::PrototypeArgument)?;
            let destination_socket =
                graph.source(prototype, EdgeWeightKindDiscriminants::Prototype)?;
            let NodeWeight::InputSocket(destination_socket_node) =
                graph.get_node_weight(destination_socket)?
            else {
                return Err(ConnectionUnmigrateableBecause::DestinationIsNotInputSocket);
            };
            destination_socket_node.id().into()
        };

        // Get the source and destination components from the APA node
        //
        //     Connection APA --|.targets|-> Source Component
        //                    --|.targets|-> Dest Component
        //
        let Some(ArgumentTargets {
            source_component_id,
            destination_component_id,
        }) = graph
            .get_node_weight(apa)?
            .get_attribute_prototype_argument_node_weight()?
            .targets()
        else {
            return Err(ConnectionUnmigrateableBecause::NoArgumentTargets);
        };

        Ok(Self {
            from: (source_component_id, source_socket_id),
            to: (destination_component_id, destination_socket_id),
        })
    }

    /// If this is a connection edge, get connection data for it
    ///
    ///     Dest Socket --|Prototype|-> . --|PrototypeArgument|-> Connection APA --|PrototypeArgumentValue|-> Source Socket
    ///                                                                          --|.targets|-> Source Component
    ///                                                                          --|.targets|-> Dest Component
    ///
    /// Returns `None` if it's not a connection edge:
    ///
    ///     AttributePrototypeArgument --|PrototypeArgumentValue|-> OutputSocket
    ///
    pub fn socket_connection_edge_opt(
        graph: &WorkspaceSnapshotGraphVCurrent,
        (edge, source, target): (&EdgeWeight, NodeIndex, NodeIndex),
    ) -> Option<(AttributePrototypeArgumentId, OutputSocketId)> {
        if &EdgeWeightKind::PrototypeArgumentValue != edge.kind() {
            return None;
        }

        let apa_id = {
            let Some(NodeWeight::AttributePrototypeArgument(apa_node)) =
                graph.get_node_weight_opt(source)
            else {
                return None;
            };
            apa_node.id().into()
        };

        let output_socket_id = {
            let Some(NodeWeight::Content(output_socket_node)) = graph.get_node_weight_opt(target)
            else {
                return None;
            };
            let ContentAddress::OutputSocket(_) = output_socket_node.content_address() else {
                return None;
            };
            output_socket_node.id().into()
        };

        Some((apa_id, output_socket_id))
    }

    pub async fn fmt_title(
        &self,
        ctx: &DalContext,
        f: &mut impl std::fmt::Write,
    ) -> std::fmt::Result {
        let from_variant = match Component::schema_variant_id(ctx, self.from.0).await {
            Ok(variant_id) => SchemaVariant::fmt_title(ctx, variant_id).await,
            Err(err) => err.to_string(),
        };
        let to_variant = match Component::schema_variant_id(ctx, self.to.0).await {
            Ok(variant_id) => SchemaVariant::fmt_title(ctx, variant_id).await,
            Err(err) => err.to_string(),
        };

        write!(
            f,
            "socket connection {} on {} --> {} on {}",
            OutputSocket::fmt_title(ctx, self.from.1).await,
            from_variant,
            InputSocket::fmt_title(ctx, self.to.1).await,
            to_variant,
        )
    }
}

///
/// A prop connection (subscription) definition.
///
///     *Dest AV*      --|Prototype|-> . --|PrototypeArgument|-> . --|ValueSubscription(*Source Prop* Path)|-> . <-|Root|-- Source Component
///                                                              . --|Use|-> *Func Argument*
///                                    . --|Use|-> *Func*
///
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct PropConnection {
    pub from: (ComponentId, jsonptr::PointerBuf),
    pub to: (ComponentId, jsonptr::PointerBuf),
    pub func_id: FuncId,
}

impl PropConnection {
    /// Get the equivalent PropConnection for this connection, or report an error if it can't be
    /// migrated.
    ///
    /// - The connection must connect a socket to a prop:
    ///
    ///       Dest Socket --|Prototype|-> . --|PrototypeArgument|-> *Connection APA* --|PrototypeArgumentValue|-> Source Socket
    ///                                                                              --|.targets|-> Source Component
    ///                                                                              --|.targets|-> Dest Component
    ///
    /// - A single argument binding the Source Socket to a Source Prop:
    ///
    ///       Source Socket --|Prototype|-> . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> Source Prop
    ///                                                               . --|Use|-> *Func Argument*
    ///                                     . --|Use|-> *Func*
    ///
    /// - Dest Props with a single argument binding each to Dest Socket:
    ///
    ///       *Dest Prop*   --|Prototype|-> . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> Dest Socket
    ///                                                               . --|Use|-> *Func Argument*
    ///                                     . --|Use|-> *Func*
    ///
    /// - A single Dest AV at the each Dest Prop's path for the given Dest Component. No map or array items on the path.
    ///
    ///       Source Component --|Root|-> . --|Contains|-...-> *Dest AV* --|Prop|-> Dest Prop
    ///
    /// - Either the source or destination func must be identity (the other func+argument will be used)
    /// - There must not be multiple connections to the same destination AV (enforced elsewhere)
    ///
    fn equivalent_to_socket_connection(
        graph: &WorkspaceSnapshotGraphVCurrent,
        &SocketConnection {
            from: (source_component_id, source_socket_id),
            to: (destination_component_id, destination_socket_id),
        }: &SocketConnection,
    ) -> Result<Vec<Self>, ConnectionUnmigrateableBecause> {
        // Make sure the sockets have the same schema as their component (multiplayer issue)
        //
        //     Socket <-|Socket|-- SchemaVariant
        //     Component --|Use|-> SchemaVariant
        {
            let component = graph.get_node_index_by_id(source_component_id)?;
            let schema_variant = graph.target(component, EdgeWeightKindDiscriminants::Use)?;
            let socket = graph.get_node_index_by_id(source_socket_id)?;
            if !graph
                .sources(socket, EdgeWeightKind::Socket)
                .any(|source| source == schema_variant)
            {
                return Err(ConnectionUnmigrateableBecause::SourceSocketSchemaMismatch);
            }
        }
        {
            let component = graph.get_node_index_by_id(destination_component_id)?;
            let socket = graph.get_node_index_by_id(destination_socket_id)?;
            let schema_variant = graph.target(component, EdgeWeightKindDiscriminants::Use)?;
            if !graph
                .sources(socket, EdgeWeightKind::Socket)
                .any(|source| source == schema_variant)
            {
                return Err(ConnectionUnmigrateableBecause::DestinationSocketSchemaMismatch);
            }
        }

        // Get the source AV path and function for the source component+socket
        let (source_prop_id, source_func_id) = Self::source_prop(graph, source_socket_id)?;
        let (_, source_path) = prop_path_from_root(graph, source_prop_id)?;

        // Get the destination AV path and function for the destination component+socket
        let mut prop_connections = vec![];
        for (destination_prop_id, destination_func_id) in
            Self::destination_props(graph, destination_socket_id)?
        {
            let (_, mut destination_path) = prop_path_from_root(graph, destination_prop_id)?;

            // Pick the function to use for the connection (since there are two functions involved,
            // we hope one of them is identity or normalizeToArray so we can pick the other!)
            let func_id = if is_identity_func(graph, destination_func_id)? {
                source_func_id

            // Figure out whether the source will produce an array or not based on the input
            // prop and the type of the input
            } else if is_normalize_to_array_func(graph, destination_func_id)? {
                let source_is_array = PropKind::Array
                    == graph
                        .get_node_weight_by_id(source_prop_id)?
                        .get_prop_node_weight()?
                        .kind;
                match func_produces_array(graph, source_func_id, source_is_array)? {
                    // If the source function already produces an array, we can use it as is
                    Some(true) => source_func_id,

                    // If the source function produces a single value, we append a new element to the
                    // array and set the subscription on the new element.
                    Some(false) => {
                        destination_path.push_back("-");
                        source_func_id
                    }

                    // If we don't know whether the source func yields an array or not, we can't
                    // migrate a normalizeToArray connection!
                    None => {
                        return Err(
                            ConnectionUnmigrateableBecause::SourceAndDestinationSocketBothHaveFuncs {
                                source_func_id,
                                destination_func_id,
                            },
                        );
                    }
                }

            // If the source function is identity, we can use the destination function as is
            } else if is_identity_func(graph, source_func_id)? {
                destination_func_id

            // If both functions are non-identity and we couldn't figure out which to use,
            // we can't migrate the connection.
            } else {
                return Err(
                    ConnectionUnmigrateableBecause::SourceAndDestinationSocketBothHaveFuncs {
                        source_func_id,
                        destination_func_id,
                    },
                );
            };

            prop_connections.push(PropConnection {
                to: (destination_component_id, destination_path),
                from: (source_component_id, source_path.clone()),
                func_id,
            })
        }

        Ok(prop_connections)
    }

    /// Get the source prop and transformation func for a socket by looking at its bindings
    ///
    ///     Source Socket --|Prototype|-> . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> *Source Prop*
    ///                                                             . --|Use|-> *Func Argument*
    ///                                   . --|Use|-> *Func*
    ///
    fn source_prop(
        graph: &WorkspaceSnapshotGraphVCurrent,
        source_socket_id: OutputSocketId,
    ) -> Result<(PropId, FuncId), ConnectionUnmigrateableBecause> {
        // Get the prototype for the output socket
        //
        //     Source Socket --|Prototype|-> .
        //
        let source_socket = graph.get_node_index_by_id(source_socket_id)?;
        let prototype = graph.target(source_socket, EdgeWeightKindDiscriminants::Prototype)?;

        // Get the prop the prototype is bound to, and the arg through which it is bound
        //
        //    . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> *Source Prop*
        //
        let prop_id = {
            //    . --|PrototypeArgument|-> .
            let arg = {
                let mut args =
                    graph.targets(prototype, EdgeWeightKindDiscriminants::PrototypeArgument);
                // Must have an arg
                let Some(arg) = args.next() else {
                    return Err(
                        ConnectionUnmigrateableBecause::SourceSocketPrototypeHasNoArguments,
                    );
                };
                // Must be a single arg
                if args.next().is_some() {
                    return Err(
                        ConnectionUnmigrateableBecause::SourceSocketPrototypeHasMultipleArguments,
                    );
                };
                arg
            };

            let prop = graph.target(arg, EdgeWeightKind::PrototypeArgumentValue)?;
            // Must be a bound to a prop
            let NodeWeight::Prop(prop_node) = graph.get_node_weight(prop)? else {
                return Err(
                    ConnectionUnmigrateableBecause::SourceSocketPrototypeArgumentNotBoundToProp,
                );
            };
            prop_node.id().into()
        };

        // Get the func the prototype is bound to
        //
        //     . --|Use|-> *Func*
        //
        let func_id = {
            let func = graph.target(prototype, EdgeWeightKindDiscriminants::Use)?;
            graph
                .get_node_weight(func)?
                .get_func_node_weight()?
                .id()
                .into()
        };

        Ok((prop_id, func_id))
    }

    /// Get the destination props and transformation funcs for a socket by looking at its bindings
    ///
    ///     *Dest Prop* --|Prototype|-> . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> Dest Socket
    ///                                 . --|Use|-> *Func*
    ///
    /// - All prototypes pointing at this socket must be be for props
    /// - All prop prototypes must have a single argument
    fn destination_props(
        graph: &WorkspaceSnapshotGraphVCurrent,
        destination_socket_id: InputSocketId,
    ) -> Result<Vec<(PropId, FuncId)>, ConnectionUnmigrateableBecause> {
        // Get the prototype the input socket is bound to, and the argument through which it is bound
        //
        //     . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> Source Socket
        //
        let destination_socket = graph.get_node_index_by_id(destination_socket_id)?;
        let mut result = vec![];
        for arg in graph.sources(destination_socket, EdgeWeightKind::PrototypeArgumentValue) {
            let prototype = graph.source(arg, EdgeWeightKind::PrototypeArgument)?;

            // Get the prop the prototype is bound to
            let destination_prop_id = {
                let prop = graph.source(prototype, EdgeWeightKindDiscriminants::Prototype)?;
                let NodeWeight::Prop(prop_node) = graph.get_node_weight(prop)? else {
                    return Err(
                        ConnectionUnmigrateableBecause::DestinationSocketArgumentNotBoundToProp,
                    );
                };
                prop_node.id().into()
            };

            // Get the func the prototype calls
            let destination_func_id = {
                let func = graph.target(prototype, EdgeWeightKindDiscriminants::Use)?;
                graph
                    .get_node_weight(func)?
                    .get_func_node_weight()?
                    .id()
                    .into()
            };

            // Error if there are any other arguments; right now we don't support that
            if graph
                .targets(prototype, EdgeWeightKindDiscriminants::PrototypeArgument)
                .count()
                > 1
            {
                return Err(
                    ConnectionUnmigrateableBecause::DestinationPrototypeHasMultipleArgs {
                        destination_prop_id,
                        destination_func_id,
                    },
                );
            }

            result.push((destination_prop_id, destination_func_id));
        }

        Ok(result)
    }

    pub async fn fmt_title(
        &self,
        ctx: &DalContext,
        f: &mut impl std::fmt::Write,
    ) -> std::fmt::Result {
        match Func::intrinsic_kind(ctx, self.func_id).await {
            // Don't print si:identity, that's standard
            Ok(Some(IntrinsicFunc::Identity)) => write!(f, "{}", &self.from.1)?,
            _ => {
                let func = Func::fmt_title(ctx, self.func_id).await;
                write!(f, "{}({})", func, &self.from.1)?
            }
        }
        write!(f, " --> {}", &self.to.1)
    }
}

/// The reason we can't migrate a socket connection
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, strum::EnumDiscriminants)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ConnectionUnmigrateableBecause {
    /// The connection prototype has multiple argument values, so can't be safely migrated to a
    /// single prop connection
    ConnectionPrototypeHasMultipleArgs,
    /// The connection APA is hanging off something other than an input socket
    DestinationIsNotInputSocket,
    /// The destination socket is bound to more than one argument, prototype, or prop
    DestinationPrototypeHasMultipleArgs {
        destination_prop_id: PropId,
        destination_func_id: FuncId,
    },
    /// The destination socket is not bound to a prop
    DestinationSocketArgumentNotBoundToProp,
    /// The destination socket is bound to a prop, but we can't find the AV for it
    DestinationSocketBoundToPropWithNoValue { destination_prop_id: PropId },
    /// The destination socket comes from the wrong schema
    DestinationSocketSchemaMismatch,
    /// The connection exists, but the graph is invalid in some unexpected way that would
    /// break its ability to be used (e.g. component_id points to missing component node)
    InvalidGraph { error: String },
    /// Multiple connections all hook up to the same destination, so we can't migrate any of them
    MultipleConnectionsToSameProp,
    /// The connection prototype has no source or target component IDs
    NoArgumentTargets,
    /// Both the destination and source sockets have non-identity functions connecting them to
    /// their props, so we can't pick a single function to use for the new connection
    SourceAndDestinationSocketBothHaveFuncs {
        source_func_id: FuncId,
        destination_func_id: FuncId,
    },
    /// There is a single binding for this socket, but it is not bound to a prop
    SourceSocketPrototypeArgumentNotBoundToProp,
    /// There are multiple arguments on the source socket, so we can't pick a single one to migrate to
    SourceSocketPrototypeHasMultipleArguments,
    /// There are no arguments on the source socket, so we can't pick a single one to migrate to
    SourceSocketPrototypeHasNoArguments,
    /// The source socket comes from the wrong schema
    SourceSocketSchemaMismatch,
}

impl ConnectionUnmigrateableBecause {
    pub async fn fmt_title(
        &self,
        ctx: &DalContext,
        f: &mut impl std::fmt::Write,
    ) -> std::fmt::Result {
        match *self {
            ConnectionUnmigrateableBecause::ConnectionPrototypeHasMultipleArgs => {
                write!(f, "Connection prototype has multiple arguments")
            }
            ConnectionUnmigrateableBecause::DestinationIsNotInputSocket => {
                write!(f, "Destination is not an input socket")
            }
            ConnectionUnmigrateableBecause::DestinationSocketArgumentNotBoundToProp => {
                write!(f, "Destination socket argument is not bound to a prop")
            }
            ConnectionUnmigrateableBecause::DestinationSocketBoundToPropWithNoValue {
                destination_prop_id,
            } => {
                write!(
                    f,
                    "Destination socket is bound to a prop with no value: {}",
                    Prop::fmt_title(ctx, destination_prop_id).await
                )
            }
            ConnectionUnmigrateableBecause::DestinationSocketSchemaMismatch => {
                write!(
                    f,
                    "Destination socket comes from a different schema than the destination component (multiplayer issue)"
                )
            }
            ConnectionUnmigrateableBecause::DestinationPrototypeHasMultipleArgs {
                destination_func_id,
                destination_prop_id,
            } => {
                write!(
                    f,
                    "Destination prop {} passes multiple args to function {}",
                    Prop::fmt_title(ctx, destination_prop_id).await,
                    Func::fmt_title(ctx, destination_func_id).await,
                )
            }
            ConnectionUnmigrateableBecause::InvalidGraph { ref error } => {
                write!(f, "Invalid graph: {error}")
            }
            ConnectionUnmigrateableBecause::MultipleConnectionsToSameProp => {
                write!(f, "Multiple connections to the same prop")
            }
            ConnectionUnmigrateableBecause::NoArgumentTargets => {
                write!(
                    f,
                    "Connection prototype has no source or destination component IDs"
                )
            }
            ConnectionUnmigrateableBecause::SourceAndDestinationSocketBothHaveFuncs {
                source_func_id,
                destination_func_id: dest_func_id,
            } => {
                write!(
                    f,
                    "Source and destination sockets both have non-identity functions: {} and {}",
                    Func::fmt_title(ctx, dest_func_id).await,
                    Func::fmt_title(ctx, source_func_id).await,
                )
            }
            ConnectionUnmigrateableBecause::SourceSocketPrototypeArgumentNotBoundToProp => {
                write!(f, "Source socket prototype argument is not bound to a prop")
            }
            ConnectionUnmigrateableBecause::SourceSocketPrototypeHasMultipleArguments => {
                write!(f, "Source socket prototype has multiple arguments")
            }
            ConnectionUnmigrateableBecause::SourceSocketPrototypeHasNoArguments => {
                write!(f, "Source socket prototype has no arguments")
            }
            ConnectionUnmigrateableBecause::SourceSocketSchemaMismatch => {
                write!(
                    f,
                    "Source socket comes from a different schema than the source component (multiplayer issue)"
                )
            }
        }
    }
}

impl From<WorkspaceSnapshotGraphError> for ConnectionUnmigrateableBecause {
    fn from(err: WorkspaceSnapshotGraphError) -> Self {
        Self::InvalidGraph {
            error: err.to_string(),
        }
    }
}
impl From<NodeWeightError> for ConnectionUnmigrateableBecause {
    fn from(err: NodeWeightError) -> Self {
        Self::InvalidGraph {
            error: err.to_string(),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, &'_ ConnectionMigration> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, migration) = self;
        if let Some(ref issue) = migration.issue {
            write!(f, "ERROR {}: ", WithGraph(graph, issue))?;
        }
        write!(f, "migrate")?;
        if let Some(ref socket_connection) = migration.socket_connection {
            write!(f, " {}", WithGraph(graph, socket_connection))?;
            for prop_connection in &migration.prop_connections {
                write!(f, " to {}", WithGraph(graph, prop_connection))?;
            }
        }

        match migration.explicit_connection_id {
            Some(explicit_connection_id) => {
                write!(f, " (explicit connection APA {explicit_connection_id})")
            }
            None => write!(f, " (inferred connection)"),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, &'_ SocketConnection> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, socket_connection) = self;
        write!(
            f,
            "{} on component {} --> {} on component {}",
            WithGraph(graph, socket_connection.from.1),
            WithGraph(graph, socket_connection.from.0),
            WithGraph(graph, socket_connection.to.1),
            WithGraph(graph, socket_connection.to.0),
        )
    }
}

impl std::fmt::Display for WithGraph<'_, &'_ (ComponentId, jsonptr::PointerBuf)> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, &(component_id, ref path)) = self;
        write!(f, "{} on {}", WithGraph(graph, component_id), path)
    }
}

impl std::fmt::Display for WithGraph<'_, &'_ PropConnection> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, prop_connection) = self;
        write!(
            f,
            "prop connection {} -> {}",
            WithGraph(graph, &prop_connection.from),
            WithGraph(graph, &prop_connection.to)
        )?;
        if !is_identity_func(graph, prop_connection.func_id).unwrap_or(true) {
            write!(f, " using {}", WithGraph(graph, prop_connection.func_id))?;
        }
        Ok(())
    }
}

impl std::fmt::Display for WithGraph<'_, &'_ ConnectionUnmigrateableBecause> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, because) = self;
        match *because {
            ConnectionUnmigrateableBecause::ConnectionPrototypeHasMultipleArgs => {
                write!(f, "Connection prototype has multiple arguments")
            }
            ConnectionUnmigrateableBecause::DestinationIsNotInputSocket => {
                write!(f, "Destination is not an input socket")
            }
            ConnectionUnmigrateableBecause::DestinationSocketArgumentNotBoundToProp => {
                write!(f, "Destination socket argument is not bound to a prop")
            }
            ConnectionUnmigrateableBecause::DestinationSocketBoundToPropWithNoValue {
                destination_prop_id,
            } => {
                write!(
                    f,
                    "Destination socket is bound to a prop with no value: {}",
                    WithGraph(graph, destination_prop_id)
                )
            }
            ConnectionUnmigrateableBecause::DestinationPrototypeHasMultipleArgs {
                destination_func_id,
                destination_prop_id,
            } => {
                write!(
                    f,
                    "Destination prop {} passes multiple args to function {}",
                    WithGraph(graph, destination_prop_id),
                    WithGraph(graph, destination_func_id)
                )
            }
            ConnectionUnmigrateableBecause::DestinationSocketSchemaMismatch => {
                write!(
                    f,
                    "Destination socket comes from a different schema than the destination component (multiplayer issue)"
                )
            }
            ConnectionUnmigrateableBecause::InvalidGraph { ref error } => {
                write!(f, "Invalid graph: {error}")
            }
            ConnectionUnmigrateableBecause::MultipleConnectionsToSameProp => {
                write!(f, "Multiple connections to the same prop")
            }
            ConnectionUnmigrateableBecause::NoArgumentTargets => {
                write!(
                    f,
                    "Connection prototype has no source or destination component IDs"
                )
            }
            ConnectionUnmigrateableBecause::SourceAndDestinationSocketBothHaveFuncs {
                source_func_id,
                destination_func_id: dest_func_id,
            } => {
                write!(
                    f,
                    "Source and destination sockets both have non-identity functions: {} and {}",
                    WithGraph(graph, dest_func_id),
                    WithGraph(graph, source_func_id)
                )
            }
            ConnectionUnmigrateableBecause::SourceSocketPrototypeArgumentNotBoundToProp => {
                write!(f, "Source socket prototype argument is not bound to a prop")
            }
            ConnectionUnmigrateableBecause::SourceSocketPrototypeHasMultipleArguments => {
                write!(f, "Source socket prototype has multiple arguments")
            }
            ConnectionUnmigrateableBecause::SourceSocketPrototypeHasNoArguments => {
                write!(f, "Source socket prototype has no arguments")
            }
            ConnectionUnmigrateableBecause::SourceSocketSchemaMismatch => {
                write!(
                    f,
                    "Source socket comes from a different schema than the source component (multiplayer issue)"
                )
            }
        }
    }
}

/// Sent when a connection migration is started.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMigrationStartedPayload {
    pub dry_run: bool,
}

/// Sent when a connection migration is completed. If it was a dry run or has an error, the
/// data will not be committed.
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMigrationFinishedPayload {
    pub dry_run: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(flatten)]
    pub summary: ConnectionMigrationSummary,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMigrationSummary {
    pub connections: usize,
    pub migrated: usize,
    pub unmigrateable: usize,
    pub removed_parents: usize,
}

/// Sent when a connection migration is run (though it may not actually be migrated; see
/// the "migrated" field).
pub type ConnectionMigratedPayload = ConnectionMigrationWithMessage;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionMigrationWithMessage {
    #[serde(flatten)]
    pub connection: ConnectionMigration,
    pub message: String,
}

impl WsEvent {
    pub async fn connection_migration_started(
        ctx: &DalContext,
        dry_run: bool,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ConnectionMigrationStarted(ConnectionMigrationStartedPayload { dry_run }),
        )
        .await
    }

    pub async fn connection_migration_finished(
        ctx: &DalContext,
        dry_run: bool,
        error: Option<String>,
        summary: ConnectionMigrationSummary,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ConnectionMigrationFinished(ConnectionMigrationFinishedPayload {
                dry_run,
                error,
                summary,
            }),
        )
        .await
    }

    pub async fn connection_migrated(
        ctx: &DalContext,
        migration: ConnectionMigrationWithMessage,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ConnectionMigrated(migration)).await
    }
}
