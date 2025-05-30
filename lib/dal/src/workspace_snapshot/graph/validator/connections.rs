use std::collections::HashSet;

use itertools::Itertools;
use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    AttributePrototypeArgumentId,
    AttributeValueId,
    ComponentId,
    FuncArgumentId,
    FuncId,
    InputSocketId,
    OutputSocketId,
    PropId,
};

use super::{
    WorkspaceSnapshotGraphError,
    av_for_path,
    is_identity_func,
    prop_path_from_root,
};
use crate::workspace_snapshot::{
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
    let graph = graph.deref();
    let inferred_connection_migrations =
        inferred_connections.into_iter().map(|socket_connection| {
            match PropConnection::equivalent_to_socket_connection(graph, &socket_connection) {
                Ok(prop_connection) => ConnectionMigration {
                    explicit_connection_id: None,
                    socket_connection: Some(socket_connection),
                    prop_connection: Some(prop_connection),
                    issue: None,
                },
                Err(issue) => ConnectionMigration {
                    explicit_connection_id: None,
                    socket_connection: Some(socket_connection),
                    prop_connection: None,
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
    let mut seen_destination_sockets = HashSet::new();
    let mut dup_destination_sockets = HashSet::new();
    for migration in &migrations {
        if let Some(ref socket_connection) = migration.socket_connection {
            if !seen_destination_sockets.insert(socket_connection.destination) {
                dup_destination_sockets.insert(socket_connection.destination);
            }
        }
    }

    // If we have multiple connections to the same destination socket, we can't migrate any of them
    for migration in &mut migrations {
        if let Some(ref socket_connection) = migration.socket_connection {
            if dup_destination_sockets.contains(&socket_connection.destination) {
                // We dedup whether or not socket connections are migrateable; if there are 10
                // un-migrateable connections to the same socket, and one migrateable one, we
                // still can't migrate any of them because it would still be inaccurate!
                migration.issue =
                    Some(ConnectionUnmigrateableBecause::MultipleConnectionsToSameSocket);
            }
        }
    }

    // TODO what if one socket sets /domain/Foo, and another sets /domain/Foo/Bar?
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
    /// The prop connection we will migrate to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prop_connection: Option<PropConnection>,
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
            prop_connection: None,
            issue: None,
        };
        match SocketConnection::from_socket_connection_edge(graph, apa_id, source_socket_id) {
            Ok(socket_connection) => {
                match PropConnection::equivalent_to_socket_connection(graph, &socket_connection) {
                    Ok(prop_connection) => migration.prop_connection = Some(prop_connection),
                    Err(issue) => migration.issue = Some(issue),
                }
                migration.socket_connection = Some(socket_connection);
            }
            Err(issue) => migration.issue = Some(issue),
        }
        migration
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
    pub source: (ComponentId, OutputSocketId),
    pub destination: (ComponentId, InputSocketId),
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
            source: (source_component_id, source_socket_id),
            destination: (destination_component_id, destination_socket_id),
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
    pub dest_av_id: AttributeValueId,
    pub source_root_av_id: AttributeValueId,
    pub source_path: jsonptr::PointerBuf,
    pub func_id: FuncId,
    pub func_arg_id: FuncArgumentId,
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
    /// - A single Dest Prop with a single argument binding it to Dest Socket:
    ///
    ///       *Dest Prop*   --|Prototype|-> . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> Dest Socket
    ///                                                               . --|Use|-> *Func Argument*
    ///                                     . --|Use|-> *Func*
    ///
    /// - A single Dest AV at the Prop's path for the given Dest Component. No map or array items on the path.
    ///
    ///       Source Component --|Root|-> . --|Contains|-...-> *Dest AV* --|Prop|-> Dest Prop
    ///
    /// - Either the source or destination func must be identity (the other func+argument will be used)
    /// - There must not be multiple connections to the same destination AV (enforced elsewhere)
    ///
    fn equivalent_to_socket_connection(
        graph: &WorkspaceSnapshotGraphVCurrent,
        socket_connection: &SocketConnection,
    ) -> Result<Self, ConnectionUnmigrateableBecause> {
        // Get the source AV path and function for the source component+socket
        let (source_root_av_id, source_path, source_func_id, source_func_arg_id) = {
            let (source_prop_id, source_func_id, source_func_arg_id) =
                Self::source_prop(graph, socket_connection.source.1)?;
            let (_, source_path) = prop_path_from_root(graph, source_prop_id)?;

            let source_component = graph.get_node_index_by_id(socket_connection.source.0)?;
            let source_root_av = graph.target(source_component, EdgeWeightKind::Root)?;
            let source_root_av_id = graph
                .get_node_weight(source_root_av)?
                .get_attribute_value_node_weight()?
                .id()
                .into();
            (
                source_root_av_id,
                source_path,
                source_func_id,
                source_func_arg_id,
            )
        };

        // Get the destination AV for the given destination component+socket
        let (dest_av_id, dest_func_id, dest_func_arg_id) = {
            let (dest_prop_id, dest_func_id, dest_func_arg_id) =
                Self::dest_prop(graph, socket_connection.destination.1)?;
            let (_, dest_path) = prop_path_from_root(graph, dest_prop_id)?;

            let destination_component =
                graph.get_node_index_by_id(socket_connection.destination.0)?;
            let dest_root_av = graph.target(destination_component, EdgeWeightKind::Root)?;
            let dest_root_av_id = graph
                .get_node_weight(dest_root_av)?
                .get_attribute_value_node_weight()?
                .id()
                .into();
            let Some(dest_av_id) = av_for_path(graph, dest_root_av_id, &dest_path)? else {
                return Err(
                    ConnectionUnmigrateableBecause::DestinationSocketBoundToPropWithNoValue {
                        dest_prop_id,
                    },
                );
            };
            (dest_av_id, dest_func_id, dest_func_arg_id)
        };

        // Figure out the func to use
        let (func_id, func_arg_id) = if is_identity_func(graph, source_func_id)? {
            (dest_func_id, dest_func_arg_id)
        } else if is_identity_func(graph, dest_func_id)? {
            (source_func_id, source_func_arg_id)
        } else {
            return Err(
                ConnectionUnmigrateableBecause::SourceAndDestinationSocketBothHaveFuncs {
                    source_func_id,
                    dest_func_id,
                },
            );
        };

        Ok(PropConnection {
            dest_av_id,
            source_root_av_id,
            source_path,
            func_id,
            func_arg_id,
        })
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
    ) -> Result<(PropId, FuncId, FuncArgumentId), ConnectionUnmigrateableBecause> {
        // Get the prototype for the output socket
        //
        //     Source Socket --|Prototype|-> .
        //
        let source_socket = graph.get_node_index_by_id(source_socket_id)?;
        let prototype = graph.target(source_socket, EdgeWeightKindDiscriminants::Prototype)?;

        // Get the prop the prototype is bound to, and the arg through which it is bound
        //
        //    . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> *Source Prop*
        //                              . --|Use|-> *Func Argument*
        //
        let (prop_id, func_arg_id) = {
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

            //    . --|PrototypeArgumentValue|-> *Source Prop*
            let prop_id = {
                let prop = graph.target(arg, EdgeWeightKind::PrototypeArgumentValue)?;
                // Must be a bound to a prop
                let NodeWeight::Prop(prop_node) = graph.get_node_weight(prop)? else {
                    return Err(
                        ConnectionUnmigrateableBecause::SourceSocketPrototypeArgumentNotBoundToProp,
                    );
                };
                prop_node.id().into()
            };

            //    . --|Use|-> *Func Argument*
            let func_arg_id = {
                let func_arg = graph.target(arg, EdgeWeightKindDiscriminants::Use)?;
                graph
                    .get_node_weight(func_arg)?
                    .get_func_argument_node_weight()?
                    .id()
                    .into()
            };

            (prop_id, func_arg_id)
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

        Ok((prop_id, func_id, func_arg_id))
    }

    /// Get the destination prop and transformation func for a socket by looking at its bindings
    ///
    ///     *Dest Prop* --|Prototype|-> . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> Dest Socket
    ///                                                           . --|Use|-> *Func Argument*
    ///                                 . --|Use|-> *Func*
    ///
    fn dest_prop(
        graph: &WorkspaceSnapshotGraphVCurrent,
        destination_socket_id: InputSocketId,
    ) -> Result<(PropId, FuncId, FuncArgumentId), ConnectionUnmigrateableBecause> {
        // Get the prototype the input socket is bound to, and the argument through which it is bound
        //
        //     . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> Source Socket
        //                               . --|Use|-> *Func Argument*
        //
        let destination_socket = graph.get_node_index_by_id(destination_socket_id)?;
        let (func_arg_id, prototype) = {
            //     . --|PrototypeArgumentValue|-> Source Socket
            let arg = {
                let mut args =
                    graph.sources(destination_socket, EdgeWeightKind::PrototypeArgumentValue);
                // Must have an arg bound to it
                let Some(arg) = args.next() else {
                    return Err(ConnectionUnmigrateableBecause::DestinationSocketHasNoBindings);
                };
                // Must be bound to a single arg
                if args.next().is_some() {
                    return Err(
                        ConnectionUnmigrateableBecause::DestinationSocketHasMultipleBindings,
                    );
                };
                arg
            };

            //     . --|Use|-> *Func Argument*
            let func_arg = graph.target(arg, EdgeWeightKindDiscriminants::Use)?;
            let func_arg_id = graph
                .get_node_weight(func_arg)?
                .get_func_argument_node_weight()?
                .id()
                .into();

            //     . --|PrototypeArgument|-> .
            let prototype = graph.source(arg, EdgeWeightKind::PrototypeArgument)?;

            (func_arg_id, prototype)
        };

        // Get the prop the prototype is bound to
        let prop_id = {
            let prop = graph.source(prototype, EdgeWeightKindDiscriminants::Prototype)?;
            let NodeWeight::Prop(prop_node) = graph.get_node_weight(prop)? else {
                return Err(
                    ConnectionUnmigrateableBecause::DestinationSocketArgumentNotBoundToProp,
                );
            };
            prop_node.id().into()
        };

        // Get the func the prototype is bound to
        let func_id = {
            let func = graph.target(prototype, EdgeWeightKindDiscriminants::Use)?;
            graph
                .get_node_weight(func)?
                .get_func_node_weight()?
                .id()
                .into()
        };

        Ok((prop_id, func_id, func_arg_id))
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
    /// The destination socket is not bound to a prop
    DestinationSocketArgumentNotBoundToProp,
    /// The destination socket is bound to a prop, but we can't find the AV for it
    DestinationSocketBoundToPropWithNoValue { dest_prop_id: PropId },
    /// The destination socket is bound to more than one argument, prototype, or prop
    DestinationSocketHasMultipleBindings,
    /// The destination socket is not bound to anything
    DestinationSocketHasNoBindings,
    /// The connection exists, but the graph is invalid in some unexpected way that would
    /// break its ability to be used (e.g. component_id points to missing component node)
    InvalidGraph { error: String },
    /// Multiple connections all hook up to the same destination, so we can't migrate any of them
    MultipleConnectionsToSameSocket,
    /// The connection prototype has no source or target component IDs
    NoArgumentTargets,
    /// Both the destination and source sockets have non-identity functions connecting them to
    /// their props, so we can't pick a single function to use for the new connection
    SourceAndDestinationSocketBothHaveFuncs {
        source_func_id: FuncId,
        dest_func_id: FuncId,
    },
    /// There is a single binding for this socket, but it is not bound to a prop
    SourceSocketPrototypeArgumentNotBoundToProp,
    /// There are multiple arguments on the source socket, so we can't pick a single one to migrate to
    SourceSocketPrototypeHasMultipleArguments,
    /// There are no arguments on the source socket, so we can't pick a single one to migrate to
    SourceSocketPrototypeHasNoArguments,
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
        if migration.explicit_connection_id.is_none() {
            write!(f, " inferred connection")?;
        }
        if let Some(ref socket_connection) = migration.socket_connection {
            write!(f, " {}", WithGraph(graph, socket_connection))?;
            if let Some(ref prop_connection) = migration.prop_connection {
                write!(f, " to {}", WithGraph(graph, prop_connection))?;
            }
        } else if let Some(explicit_connection_id) = migration.explicit_connection_id {
            write!(f, " connection APA {}", explicit_connection_id)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for WithGraph<'_, &'_ SocketConnection> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, socket_connection) = self;
        write!(
            f,
            "socket connection from output {} on component {} to input {} on component {}",
            WithGraph(graph, socket_connection.source.1),
            WithGraph(graph, socket_connection.source.0),
            WithGraph(graph, socket_connection.destination.1),
            WithGraph(graph, socket_connection.destination.0),
        )
    }
}

impl std::fmt::Display for WithGraph<'_, &'_ PropConnection> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, prop_connection) = self;
        write!(
            f,
            "Prop connection from {} on {} to {}",
            prop_connection.source_path,
            WithGraph(graph, prop_connection.source_root_av_id),
            WithGraph(graph, prop_connection.dest_av_id),
        )
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
                dest_prop_id,
            } => {
                write!(
                    f,
                    "Destination socket is bound to a prop with no value: {}",
                    WithGraph(graph, dest_prop_id)
                )
            }
            ConnectionUnmigrateableBecause::DestinationSocketHasMultipleBindings => {
                write!(f, "Destination socket has multiple bindings")
            }
            ConnectionUnmigrateableBecause::DestinationSocketHasNoBindings => {
                write!(f, "Destination socket has no bindings")
            }
            ConnectionUnmigrateableBecause::InvalidGraph { ref error } => {
                write!(f, "Invalid graph: {}", error)
            }
            ConnectionUnmigrateableBecause::MultipleConnectionsToSameSocket => {
                write!(f, "Multiple connections to the same socket")
            }
            ConnectionUnmigrateableBecause::NoArgumentTargets => {
                write!(
                    f,
                    "Connection prototype has no source or destination component IDs"
                )
            }
            ConnectionUnmigrateableBecause::SourceAndDestinationSocketBothHaveFuncs {
                source_func_id,
                dest_func_id,
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
        }
    }
}
