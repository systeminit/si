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
    AttributeValueId,
    ComponentId,
    FuncId,
    InputSocketId,
    OutputSocketId,
    PropId,
};

use super::{
    WorkspaceSnapshotGraphError,
    is_identity_func,
    is_normalize_to_array_func,
    prop_path_from_root,
};
use crate::{
    PropKind,
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

    // Check if any proposed prop connections already have values (which we don't want to overwrite)
    for migration in &mut migrations {
        if let Some(ref prop_connection) = migration.prop_connection {
            if let Ok(Some(av)) =
                super::resolve_av(graph, prop_connection.to.0, &prop_connection.to.1)
            {
                if has_prototype(graph, av) {
                    migration.issue =
                        Some(ConnectionUnmigrateableBecause::DestinationPropAlreadyHasValue);
                }
            }
        }
    }

    // Look for multiple connections to the same destination socket
    let mut seen_destination_sockets = HashMap::new();
    let mut dup_destination_sockets = HashSet::new();
    for migration in &migrations {
        if let Some(ref socket_connection) = migration.socket_connection {
            let is_append = migration
                .prop_connection
                .as_ref()
                .is_some_and(|prop_connection| {
                    prop_connection
                        .to
                        .1
                        .back()
                        .is_some_and(|token| token.is_next())
                });
            // If we had a previous connection to this socket, mark it as a duplicate
            if let Some(old_is_append) =
                seen_destination_sockets.insert(socket_connection.to, is_append)
            {
                // If all of them are appends, it's not a problem.
                if !(old_is_append && is_append) {
                    dup_destination_sockets.insert(socket_connection.to);
                }
            }
        }
    }

    // If we have multiple connections to the same destination socket, we can't migrate any of them
    for migration in &mut migrations {
        if let Some(ref socket_connection) = migration.socket_connection {
            if dup_destination_sockets.contains(&socket_connection.to) {
                // We dedup whether or not socket connections are migrateable; if there are 10
                // un-migrateable connections to the same socket, and one migrateable one, we
                // still can't migrate any of them because it would still be inaccurate!
                migration.issue =
                    Some(ConnectionUnmigrateableBecause::MultipleConnectionsToSameSocket);
            }
        }
    }

    migrations
}

fn has_prototype(
    graph: &impl std::ops::Deref<Target = WorkspaceSnapshotGraphVCurrent>,
    av_id: AttributeValueId,
) -> bool {
    graph.get_node_index_by_id_opt(av_id).is_some_and(|av| {
        graph
            .target(av, EdgeWeightKindDiscriminants::Prototype)
            .is_ok()
    })
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
        &SocketConnection {
            from: (source_component_id, source_socket_id),
            to: (destination_component_id, destination_socket_id),
        }: &SocketConnection,
    ) -> Result<Self, ConnectionUnmigrateableBecause> {
        // Get the source AV path and function for the source component+socket
        let (source_prop_id, source_func_id) = Self::source_prop(graph, source_socket_id)?;
        let (_, source_path) = prop_path_from_root(graph, source_prop_id)?;

        // Get the destination AV path and function for the destination component+socket
        let (destination_prop_id, destination_func_id) =
            Self::dest_prop(graph, destination_socket_id)?;
        let (_, mut destination_path) = prop_path_from_root(graph, destination_prop_id)?;

        // Figure out the func to use, if one side is not identity.
        let func_id = if is_identity_func(graph, source_func_id)? {
            // If the dest func is normalizeToArray, we can safely use identity as long as we
            // uplevel the AVs instead of the functions.
            if is_normalize_to_array_func(graph, destination_func_id)? {
                // If the other side is not an array, we append a new element to the array and
                // attach the subscription to that.
                if PropKind::Array
                    != graph
                        .get_node_weight_by_id(source_prop_id)?
                        .get_prop_node_weight()?
                        .kind
                {
                    destination_path.push_back("-"); // append
                }
                source_func_id // we already checked that this is identity
            } else {
                destination_func_id
            }
        } else if is_identity_func(graph, destination_func_id)? {
            source_func_id
        } else {
            // If the dest is si:normalizeToArray
            return Err(
                ConnectionUnmigrateableBecause::SourceAndDestinationSocketBothHaveFuncs {
                    source_func_id,
                    destination_func_id,
                },
            );
        };

        Ok(PropConnection {
            to: (destination_component_id, destination_path),
            from: (source_component_id, source_path),
            func_id,
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

    /// Get the destination prop and transformation func for a socket by looking at its bindings
    ///
    ///     *Dest Prop* --|Prototype|-> . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> Dest Socket
    ///                                 . --|Use|-> *Func*
    ///
    fn dest_prop(
        graph: &WorkspaceSnapshotGraphVCurrent,
        destination_socket_id: InputSocketId,
    ) -> Result<(PropId, FuncId), ConnectionUnmigrateableBecause> {
        // Get the prototype the input socket is bound to, and the argument through which it is bound
        //
        //     . --|PrototypeArgument|-> . --|PrototypeArgumentValue|-> Source Socket
        //
        let destination_socket = graph.get_node_index_by_id(destination_socket_id)?;
        let prototype = {
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

            //     . --|PrototypeArgument|-> .
            graph.source(arg, EdgeWeightKind::PrototypeArgument)?
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

        Ok((prop_id, func_id))
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
    /// The destination prop already has a value
    DestinationPropAlreadyHasValue,
    /// The destination socket is not bound to a prop
    DestinationSocketArgumentNotBoundToProp,
    /// The destination socket is bound to a prop, but we can't find the AV for it
    DestinationSocketBoundToPropWithNoValue { destination_prop_id: PropId },
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
        destination_func_id: FuncId,
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
        if let Some(ref socket_connection) = migration.socket_connection {
            write!(f, " {}", WithGraph(graph, socket_connection))?;
            if let Some(ref prop_connection) = migration.prop_connection {
                write!(f, " to {}", WithGraph(graph, prop_connection))?;
            }
        }

        match migration.explicit_connection_id {
            Some(explicit_connection_id) => {
                write!(f, " (explicit connection APA {})", explicit_connection_id)
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
            "socket connection from output {} on component {} to input {} on component {}",
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
            ConnectionUnmigrateableBecause::DestinationPropAlreadyHasValue => {
                write!(f, "Destination prop already has a value")
            }
            ConnectionUnmigrateableBecause::DestinationSocketArgumentNotBoundToProp => {
                write!(f, "Destination socket argument is not bound to a prop")
            }
            ConnectionUnmigrateableBecause::DestinationSocketBoundToPropWithNoValue {
                destination_prop_id: dest_prop_id,
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
        }
    }
}
