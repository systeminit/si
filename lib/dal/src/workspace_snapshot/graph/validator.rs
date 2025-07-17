use std::collections::{
    HashMap,
    HashSet,
};

use itertools::Itertools as _;
use lazy_static::lazy_static;
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
    SchemaVariantId,
};

use super::WorkspaceSnapshotGraphError;
use crate::{
    prop::PropKind,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        edge_weight::{
            EdgeWeight,
            EdgeWeightKind,
            EdgeWeightKindDiscriminants,
        },
        graph::{
            WorkspaceSnapshotGraphResult,
            WorkspaceSnapshotGraphVCurrent,
        },
        node_weight::{
            ArgumentTargets,
            AttributePrototypeArgumentNodeWeight,
            AttributeValueNodeWeight,
            NodeWeight,
            traits::SiNodeWeight as _,
        },
    },
};

pub mod connections;

/// Validate the entire graph, returning a list of validation issues found.
pub fn validate_graph_with_text(
    graph: &impl std::ops::Deref<Target = WorkspaceSnapshotGraphVCurrent>,
) -> WorkspaceSnapshotGraphResult<Vec<(ValidationIssue, String)>> {
    Ok(validate_graph(graph)?
        .into_iter()
        .map(|issue| {
            let text = format!("{}", WithGraph(graph.deref(), &issue));
            (issue, text)
        })
        .collect())
}

/// Validate the entire graph, returning a list of validation issues found.
pub fn validate_graph(
    graph: &impl std::ops::Deref<Target = WorkspaceSnapshotGraphVCurrent>,
) -> WorkspaceSnapshotGraphResult<Vec<ValidationIssue>> {
    let mut issues = vec![];
    for node in graph.nodes() {
        if let Some(issue) = ValidateNode::validate_node(graph, node)? {
            issues.push(issue);
        }
    }
    Ok(issues)
}

trait ValidateNode {
    /// Validate a single node in the graph, returning a list of validation issues found.
    fn validate_node(
        graph: &WorkspaceSnapshotGraphVCurrent,
        node: (&Self, NodeIndex),
    ) -> WorkspaceSnapshotGraphResult<Option<ValidationIssue>>;
}

impl ValidateNode for NodeWeight {
    fn validate_node(
        graph: &WorkspaceSnapshotGraphVCurrent,
        (node, node_index): (&Self, NodeIndex),
    ) -> WorkspaceSnapshotGraphResult<Option<ValidationIssue>> {
        match node {
            Self::AttributeValue(node) => ValidateNode::validate_node(graph, (node, node_index)),
            Self::AttributePrototypeArgument(node) => {
                ValidateNode::validate_node(graph, (node, node_index))
            }
            _ => Ok(None),
        }
    }
}

impl ValidateNode for AttributeValueNodeWeight {
    fn validate_node(
        graph: &WorkspaceSnapshotGraphVCurrent,
        (_, attr): (&Self, NodeIndex),
    ) -> WorkspaceSnapshotGraphResult<Option<ValidationIssue>> {
        // If this is an object attribute value, check that it has child attribute values for each child prop
        let Some(prop) = graph.target_opt(attr, EdgeWeightKind::Prop)? else {
            return Ok(None);
        };
        if PropKind::Object == graph.get_node_weight(prop)?.as_prop_node_weight()?.kind() {
            // Check our the children we *do* have against the child props we *should* have
            let child_props: HashSet<_> = graph
                .targets(prop, EdgeWeightKindDiscriminants::Use)
                .collect();

            // Step through child avs, and record the ones we see (maybe report duplicates)
            let mut attr_content = HashMap::new();
            for child_attr in graph.targets(attr, EdgeWeightKindDiscriminants::Contain) {
                let child_attr_prop = graph.target(child_attr, EdgeWeightKind::Prop)?;
                if !child_props.contains(&child_attr_prop) {
                    return Ok(Some(ValidationIssue::UnknownChildAttributeValue {
                        child: graph.get_node_weight(child_attr)?.id().into(),
                    }));
                }
                let content = graph
                    .get_node_weight(child_attr)?
                    .get_attribute_value_node_weight()?
                    .value();
                if let Some(orig_content) = attr_content.insert(child_attr_prop, content) {
                    let original = graph.get_node_weight(child_attr)?.id().into();
                    let duplicate = graph.get_node_weight(child_attr)?.id().into();
                    if content == orig_content {
                        return Ok(Some(ValidationIssue::DuplicateAttributeValue {
                            original,
                            duplicate,
                        }));
                    } else {
                        return Ok(Some(
                            ValidationIssue::DuplicateAttributeValueWithDifferentValues {
                                original,
                                duplicate,
                            },
                        ));
                    };
                }
            }

            // If any child attributes are *not* associated with child props, report those
            let mut missing_children = HashSet::new();
            for child_prop in child_props {
                if !attr_content.contains_key(&child_prop) {
                    missing_children.insert(graph.get_node_weight(child_prop)?.id().into());
                }
            }
            if !missing_children.is_empty() {
                return Ok(Some(ValidationIssue::MissingChildAttributeValues {
                    object: graph.get_node_weight(attr)?.id().into(),
                    missing_children,
                }));
            }
        }

        Ok(None)
    }
}

impl ValidateNode for AttributePrototypeArgumentNodeWeight {
    fn validate_node(
        graph: &WorkspaceSnapshotGraphVCurrent,
        (apa_node, apa): (&Self, NodeIndex),
    ) -> WorkspaceSnapshotGraphResult<Option<ValidationIssue>> {
        // Check that the APA has exactly one value
        {
            let mut values = graph
                .edges_directed(apa, Direction::Outgoing)
                .filter(|edge| {
                    &EdgeWeightKind::PrototypeArgumentValue == edge.weight().kind()
                        || EdgeWeightKindDiscriminants::ValueSubscription
                            == edge.weight().kind().into()
                });
            // If there are no values, report a missing value
            if values.next().is_none() {
                return Ok(Some(ValidationIssue::MissingValue {
                    apa: graph.get_node_weight(apa)?.id().into(),
                }));
            };
            // If there are multiple values, report a multiple values issue
            if values.next().is_some() {
                return Ok(Some(ValidationIssue::MultipleValues {
                    apa: graph.get_node_weight(apa)?.id().into(),
                }));
            }
        }

        // Check for connection to unknown socket
        if let Some(ArgumentTargets {
            source_component_id,
            ..
        }) = apa_node.targets
        {
            let source_component = graph.get_node_index_by_id(source_component_id)?;
            {
                // If this is a connection to a socket or prop, make sure the component on the other
                // end has an AV for it
                let Some(source_socket) =
                    graph.target_opt(apa, EdgeWeightKind::PrototypeArgumentValue)?
                else {
                    return Ok(None);
                };
                // Make sure the source is an output socket
                if Some(ContentAddressDiscriminants::OutputSocket)
                    != graph
                        .get_node_weight(source_socket)?
                        .content_address_discriminants()
                {
                    return Ok(None);
                };
                // Run through the sockets on the component, and find the one that matches
                let mut component_sockets = graph
                    .targets(source_component, EdgeWeightKind::SocketValue)
                    .filter_map(|socket_value| {
                        graph.target(socket_value, EdgeWeightKind::Socket).ok()
                    });
                if !component_sockets.contains(&source_socket) {
                    return Ok(Some(ValidationIssue::ConnectionToUnknownSocket {
                        apa: graph.get_node_weight(apa)?.id().into(),
                        source_component: graph.get_node_weight(source_component)?.id().into(),
                        source_socket: graph.get_node_weight(source_socket)?.id().into(),
                    }));
                }
            }
        }

        Ok(None)
    }
}

/// A single validation issue found in the graph
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ValidationIssue {
    /// APA is connected to a socket that does not exist on the source component
    ConnectionToUnknownSocket {
        apa: AttributePrototypeArgumentId,
        source_component: ComponentId,
        source_socket: OutputSocketId,
    },
    /// A child prop of an object has more than one attribute value
    DuplicateAttributeValue {
        original: AttributeValueId,
        duplicate: AttributeValueId,
    },
    /// A child prop of an object has more than one attribute value, with different values
    DuplicateAttributeValueWithDifferentValues {
        original: AttributeValueId,
        duplicate: AttributeValueId,
    },
    /// One or more child props have no corresponding attribute values
    MissingChildAttributeValues {
        object: AttributeValueId,
        missing_children: HashSet<PropId>,
    },
    /// APA has neither a PrototypeArgumentValue or ValueSubscription edge
    MissingValue { apa: AttributePrototypeArgumentId },
    /// APA has multiple PrototypeArgumentValue and ValueSubscription edges
    MultipleValues { apa: AttributePrototypeArgumentId },
    /// A child attribute value was found under an object, but it was not associated with any
    /// of the object's child props
    UnknownChildAttributeValue { child: AttributeValueId },
}

/// Helper to format values that need extra context when being displayed (such as graphs, lookup tables,
/// etc).
pub struct WithGraph<'a, T>(pub &'a WorkspaceSnapshotGraphVCurrent, pub T);

impl std::fmt::Display for WithGraph<'_, AttributeValueId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, id) = self;
        match av_path_from_root(graph, id) {
            Ok((_, path)) => write!(f, "{path} ({id})"),
            Err(err) => write!(f, "{err} ({id})"),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, AttributePrototypeArgumentId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(_, id) = self;
        write!(f, "{id}")
    }
}

impl std::fmt::Display for WithGraph<'_, ComponentId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(_, id) = self;
        write!(f, "{id}")
    }
}

impl std::fmt::Display for WithGraph<'_, FuncId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, id) = self;
        match graph
            .get_node_weight_by_id(id)
            .and_then(|node| node.get_func_node_weight().map_err(Into::into))
        {
            Ok(node) => write!(f, "{} ({id})", node.name()),
            Err(err) => write!(f, "{err} ({id})"),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, PropId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, id) = self;
        match prop_path_from_root(graph, id) {
            Ok((_, path)) => write!(f, "{path} ({id})"),
            Err(err) => write!(f, "{err} ({id})"),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, InputSocketId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(_, id) = self;
        write!(f, "{id}")
    }
}

impl std::fmt::Display for WithGraph<'_, OutputSocketId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(_, id) = self;
        write!(f, "{id}")
    }
}

impl std::fmt::Display for WithGraph<'_, &'_ ValidationIssue> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, issue) = self;
        match *issue {
            ValidationIssue::ConnectionToUnknownSocket {
                apa: dest_apa,
                source_component,
                source_socket,
            } => {
                write!(
                    f,
                    "Connection from APA {dest_apa} to unknown socket {source_socket} on component {source_component}"
                )
            }
            ValidationIssue::DuplicateAttributeValue {
                original,
                duplicate,
            } => {
                write!(
                    f,
                    "Duplicate attribute value: {} and {}",
                    WithGraph(graph, original),
                    WithGraph(graph, duplicate)
                )
            }
            ValidationIssue::DuplicateAttributeValueWithDifferentValues {
                original,
                duplicate,
            } => {
                write!(
                    f,
                    "Duplicate attribute value (with different value!): {} and {}",
                    WithGraph(graph, original),
                    WithGraph(graph, duplicate)
                )
            }
            ValidationIssue::MissingChildAttributeValues {
                object,
                ref missing_children,
            } => {
                write!(
                    f,
                    "Missing child attribute values for object {}: missing {}",
                    WithGraph(graph, object),
                    missing_children
                        .iter()
                        .map(|&child_prop| format!("{}", WithGraph(graph, child_prop)))
                        .join(", ")
                )
            }
            ValidationIssue::MissingValue { apa } => {
                write!(f, "APA {} has no value", WithGraph(graph, apa))
            }
            ValidationIssue::MultipleValues { apa } => {
                write!(f, "APA {} has multiple values", WithGraph(graph, apa))
            }
            ValidationIssue::UnknownChildAttributeValue { child } => {
                write!(
                    f,
                    "Child attribute value has unknown (non-child) prop: {}",
                    WithGraph(graph, child)
                )
            }
        }
    }
}

pub fn is_identity_func(
    graph: &WorkspaceSnapshotGraphVCurrent,
    func_id: FuncId,
) -> WorkspaceSnapshotGraphResult<bool> {
    let func_node = graph
        .get_node_weight_by_id(func_id)?
        .get_func_node_weight()?;
    Ok(IDENTITY_FUNCS.contains(func_node.name()))
}

lazy_static! {
    static ref IDENTITY_FUNCS: HashSet<&'static str> = [
        "si:identity",

        // These functions from IAM assets are essentially identity--either direct passthroughs,
        // or `value || ""`

        // AWS::IAM::CustomerManagedIdentityPolicy
        "awsIamCustomerManagedIdentityPolicyResourceArnToSocket",
        "awsIamCustomerManagedIdentityPolicySetPolicyOutput",

        // AWS::IAM::Group
        "awsIamGroupSetArnOutput",
        "awsIamGroupSetGroupNameOutput",

        // AWS::IAM::InstanceProfile
        "awsIamInstanceProfileArnOutput",
        "awsIamInstanceProfileSetRoleNameFromInput",

        // AWS::IAM::PolicyPrincipal
        "awsIamPrincipalOutputSocket",

        // AWS::IAM::Role
        "awsIamRoleSetArnOutput",
        "awsIamRoleSetRoleNameOutput",

        // AWS::IAM::RolePolicy
        "awsIamRolePolicySetPolicyArnFromSocket",
        "awsIamRolePolicySetRoleNameFromSocket",

        // AWS::IAM::User
        "awsIamUserNameToOutput",
        "awsIamUserSetOutputSocketToARN",

        // AWS::IAM::UserPolicy
        "awsIamUserPolicySetUserNameFromInput",
        "awsIamIdentityPolicyArnFromInputSocket",
    ]
    .into_iter()
    .collect();
}

pub fn is_normalize_to_array_func(
    graph: &WorkspaceSnapshotGraphVCurrent,
    func_id: FuncId,
) -> WorkspaceSnapshotGraphResult<bool> {
    let func_node = graph
        .get_node_weight_by_id(func_id)?
        .get_func_node_weight()?;
    Ok(NORMALIZE_TO_ARRAY_FUNCS.contains(func_node.name()))
}

lazy_static! {
    static ref NORMALIZE_TO_ARRAY_FUNCS: HashSet<&'static str> = [
        "si:normalizeToArray",
        "normalizeToArray",

        // These functions from IAM assets are essentially normalizeToArray by another name

        // AWS::IAM::CustomerManagedIdentityPolicy
        "awsIamCustomerManagedIdentityPolicySetStatement",

        // AWS::IAM::Group
        "awsIamGroupSetUsersFromInput",

        // AWS::IAM::PolicyStatement
        "setResourcesFromResourceSocket",
        "setNotResourcesFromNotResourceSocket",

        // AWS::IAM::Role
        "awsIamRoleSetAssumeRolePolicyStatementFromSocket",

        // AWS::IAM::RolePrincipal
        "awsIamRolePrincipalSetPrincipalOutputSocket",
    ]
    .into_iter()
    .collect();
}

pub fn func_produces_array(
    graph: &WorkspaceSnapshotGraphVCurrent,
    func_id: FuncId,
    input_is_array: bool,
) -> WorkspaceSnapshotGraphResult<Option<bool>> {
    let func_node = graph
        .get_node_weight_by_id(func_id)?
        .get_func_node_weight()?;
    Ok(
        // Identity functions preserve the input type
        if IDENTITY_FUNCS.contains(func_node.name()) {
            Some(input_is_array)

            // If the function always produces a single value, return false
        } else if SINGLE_VALUED_FUNCS.contains(func_node.name()) {
            Some(false)

        // If the function always produces an array, return true
        } else if ARRAY_VALUED_FUNCS.contains(func_node.name())
            || NORMALIZE_TO_ARRAY_FUNCS.contains(func_node.name())
        {
            Some(true)

        // Most functions we just don't know what they will do.
        // It's JavaScript, c'est la vie
        } else {
            None
        },
    )
}

lazy_static! {
    /// Known transform functions that always produce a single value, regardless of input.
    static ref SINGLE_VALUED_FUNCS: HashSet<&'static str> = [
        // AWS ARN
        "awsArnToArnString",

        // AWS::IAM::AccountPrincipal
        "awsAccountPrincipalAccountOutputSocket",
        "awsAccountPrincipalCanonicalIdOutputSocket",

        // AWS::IAM::AssumedRoleSessionPrincipal
        "awsIamAssumedRoleSessionPrincipalSetPrincipalOutputSocket",

        // AWS::IAM::ConditionOperator
        "awsIamConditionOperatorSetConditionOutput",

        // AWS::IAM::OIDCSessionPrincipal
        "awsIamOidcSessionPrincipalSetPrincipalOutputSocket",

        // AWS::IAM::PolicyStatement
        "awsIamPolicyStatementSetStatementOutput",

        // AWS::IAM::SAMLSessionPrincipal
        "awsIamSamlSessionPrincipalSetPrincipalOutputSocket",

        // AWS::IAM::ServicePrincipal
        "awsIamAwsServciePrincipalSetPrincipalOutput",
        "awsIamAwsServicePrincipalSetPrincipalOutput", // For when we fix spelling :)

        // AWS::IAM::STSFederatedUserPrincipal
        "awsIamStsFederatedUserPrincipalSetPrincipalOutput",

        // AWS::IAM::UserPrincipal
        "awsIamUserPrincipalSetPrincipalOutput",
    ]
    .into_iter()
    .collect();
}

lazy_static! {
    /// Known transform functions that always produce arrays regardless of input, but are *not*
    /// identity transforms on input elements. (si:normalizeToArray-type functions also always
    /// produce arrays, but they keep the individual elements unchanged.)
    static ref ARRAY_VALUED_FUNCS: HashSet<&'static str> = [
        // AWS::IAM::Any
        "awsIamAnyToSocket",

        // AWS::IAM::PolicySimulator
        "awsIamPolicySimulatorSetPolicyInput",
    ].into_iter().collect();
}

pub fn resolve_av(
    graph: &WorkspaceSnapshotGraphVCurrent,
    component_id: ComponentId,
    path: impl AsRef<jsonptr::Pointer>,
) -> WorkspaceSnapshotGraphResult<Option<AttributeValueId>> {
    let component = graph.get_node_index_by_id(component_id)?;
    let mut av = graph.target(component, EdgeWeightKind::Root)?;
    for segment in path.as_ref() {
        let prop = graph.target(av, EdgeWeightKind::Prop)?;
        let child_av =
            match graph.get_node_weight(prop)?.get_prop_node_weight()?.kind() {
                PropKind::Object => graph
                    .targets(av, EdgeWeightKindDiscriminants::Contain)
                    .find(|&child_av| {
                        graph
                            .target(child_av, EdgeWeightKind::Prop)
                            .and_then(|child_prop| graph.get_node_weight(child_prop))
                            .and_then(|child_prop| {
                                child_prop.get_prop_node_weight().map_err(Into::into)
                            })
                            .is_ok_and(|child_prop| child_prop.name() == segment.decoded())
                    }),
                PropKind::Map => graph
                    .edges_directed(av, Direction::Outgoing)
                    .find_map(|edge| match edge.weight().kind() {
                        EdgeWeightKind::Contain(Some(key)) if key.as_str() == segment.decoded() => {
                            Some(edge.target())
                        }
                        _ => None,
                    }),
                PropKind::Array => graph.ordered_children_for_node(av)?.and_then(|children| {
                    match segment.to_index() {
                        Ok(jsonptr::index::Index::Num(index)) => children.get(index).copied(),
                        _ => None,
                    }
                }),
                PropKind::Boolean
                | PropKind::Float
                | PropKind::Integer
                | PropKind::Json
                | PropKind::String => None,
            };
        let Some(child_av) = child_av else {
            return Ok(None);
        };
        av = child_av;
    }

    Ok(Some(
        graph
            .get_node_weight(av)?
            .get_attribute_value_node_weight()?
            .id()
            .into(),
    ))
}

pub fn av_path_from_root(
    graph: &WorkspaceSnapshotGraphVCurrent,
    id: AttributeValueId,
) -> WorkspaceSnapshotGraphResult<(AttributeValueId, jsonptr::PointerBuf)> {
    let mut index = graph.get_node_index_by_id(id)?;
    let mut path = jsonptr::PointerBuf::new();
    while let Some((
        EdgeWeight {
            kind: EdgeWeightKind::Contain(key),
        },
        parent_index,
        _,
    )) = graph
        .edges_directed_for_edge_weight_kind(
            index,
            Direction::Incoming,
            EdgeWeightKindDiscriminants::Contain,
        )
        .next()
    {
        // If we have a key, use it (must be a map)
        let parent_prop = graph
            .get_node_weight(graph.target(parent_index, EdgeWeightKind::Prop)?)?
            .get_prop_node_weight()?;
        match (parent_prop.kind(), key) {
            // Use the key if the parent is a map
            (PropKind::Map, Some(key)) => path.push_front(key),
            // Use the prop name if the parent is an object
            (PropKind::Object, None) => {
                let prop = graph
                    .get_node_weight(graph.target(index, EdgeWeightKind::Prop)?)?
                    .get_prop_node_weight()?;
                path.push_front(prop.name());
            }
            // Use the array index if the parent is an array
            (PropKind::Array, None) => match graph
                .ordered_children_for_node(parent_index)?
                .and_then(|order| order.iter().position(|&child| child == index))
            {
                Some(index) => path.push_front(index),
                // TODO return an error here instead
                None => path.push_front("<NOT IN ORDERING NODE>"),
            },

            // These are errors (and if we move this out of printing code, we should return an error)
            (PropKind::Array | PropKind::Object, Some(_)) => {
                path.push_front("<MAP KEY SPECIFIED FOR ARRAY/OBJECT>")
            }
            (PropKind::Map, None) => path.push_front("<NO MAP KEY>"),
            (kind, _) => path.push_front(format!("<BAD PARENT KIND {kind}>")),
        }
        index = parent_index;
    }

    let root_id = graph.get_node_weight(index)?.id().into();

    Ok((root_id, path))
}

pub fn prop_path_from_root(
    graph: &WorkspaceSnapshotGraphVCurrent,
    id: PropId,
) -> WorkspaceSnapshotGraphResult<(PropId, jsonptr::PointerBuf)> {
    let mut path = jsonptr::PointerBuf::new();

    let mut prop = graph.get_node_index_by_id(id)?;
    let mut prop_node = graph.get_node_weight(prop)?.get_prop_node_weight()?;
    loop {
        // If the parent is a prop, push the current node onto the path and ascend to the parent
        let parent = graph.source(prop, EdgeWeightKindDiscriminants::Use)?;
        let NodeWeight::Prop(parent_prop_node) = graph.get_node_weight(parent)? else {
            break;
        };
        path.push_front(prop_node.name());
        prop = parent;
        prop_node = parent_prop_node.clone();
    }

    let prop_id = prop_node.id().into();
    Ok((prop_id, path))
}

pub fn prop_path(
    graph: &WorkspaceSnapshotGraphVCurrent,
    id: PropId,
) -> WorkspaceSnapshotGraphResult<(SchemaVariantId, jsonptr::PointerBuf)> {
    // Get the path up to (but not including) the root prop
    let (root_prop_id, mut path) = prop_path_from_root(graph, id)?;

    // Push the root node onto the path
    let root_prop = graph.get_node_index_by_id(root_prop_id)?;
    let root_prop_node = graph.get_node_weight(root_prop)?.get_prop_node_weight()?;
    path.push_front(root_prop_node.name());

    // Get the schema variant (the parent of the root prop)
    let schema_variant = graph.source(root_prop, EdgeWeightKindDiscriminants::Use)?;
    let schema_variant_id = graph
        .get_node_weight(schema_variant)?
        .get_schema_variant_node_weight()?
        .id()
        .into();
    Ok((schema_variant_id, path))
}
