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
    ulid::Ulid,
};

use crate::{
    prop::PropKind,
    workspace_snapshot::{
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
            AttributePrototypeArgumentNodeWeight,
            AttributeValueNodeWeight,
            NodeWeight,
            OrderingNodeWeight,
            PropNodeWeight,
            traits::SiNodeWeight as _,
        },
    },
};

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
        ValidateNode::validate_node(graph, &mut issues, node)?;
    }
    Ok(issues)
}

trait ValidateNode {
    /// Validate a single node in the graph, returning a list of validation issues found.
    fn validate_node(
        graph: &WorkspaceSnapshotGraphVCurrent,
        issues: &mut Vec<ValidationIssue>,
        node: (&Self, NodeIndex),
    ) -> WorkspaceSnapshotGraphResult<()>;
}

impl ValidateNode for NodeWeight {
    fn validate_node(
        graph: &WorkspaceSnapshotGraphVCurrent,
        issues: &mut Vec<ValidationIssue>,
        (node, node_index): (&Self, NodeIndex),
    ) -> WorkspaceSnapshotGraphResult<()> {
        match node {
            Self::AttributeValue(node) => {
                ValidateNode::validate_node(graph, issues, (node, node_index))
            }
            Self::AttributePrototypeArgument(node) => {
                ValidateNode::validate_node(graph, issues, (node, node_index))
            }
            Self::Ordering(node) => ValidateNode::validate_node(graph, issues, (node, node_index)),
            Self::Prop(node) => ValidateNode::validate_node(graph, issues, (node, node_index)),
            _ => Ok(()),
        }
    }
}

impl ValidateNode for AttributePrototypeArgumentNodeWeight {
    fn validate_node(
        graph: &WorkspaceSnapshotGraphVCurrent,
        issues: &mut Vec<ValidationIssue>,
        (_, apa): (&Self, NodeIndex),
    ) -> WorkspaceSnapshotGraphResult<()> {
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
                issues.push(ValidationIssue::MissingValue {
                    apa: graph.get_node_weight(apa)?.id().into(),
                });
            };
            // If there are multiple values, report a multiple values issue
            if values.next().is_some() {
                issues.push(ValidationIssue::MultipleValues {
                    apa: graph.get_node_weight(apa)?.id().into(),
                });
            }
        }

        Ok(())
    }
}

fn validate_child_ordering(
    graph: &WorkspaceSnapshotGraphVCurrent,
    issues: &mut Vec<ValidationIssue>,
    node: NodeIndex,
    children: impl IntoIterator<Item = NodeIndex>,
) -> WorkspaceSnapshotGraphResult<()> {
    let Some(order) = graph.ordered_children_for_node(node)? else {
        issues.push(ValidationIssue::MissingOrderingNode {
            node: graph.get_node_weight(node)?.id(),
        });
        return Ok(());
    };

    let mut only_in_ordering_node: HashSet<_> = order.into_iter().collect();
    let mut only_in_children = Vec::new();
    for child in children {
        if !only_in_ordering_node.remove(&child) {
            only_in_children.push(graph.get_node_weight(child)?.id());
        }
    }

    // Report an error if there is a mismatch
    if !only_in_ordering_node.is_empty() || !only_in_children.is_empty() {
        issues.push(ValidationIssue::ChildOrderingMismatch {
            node: graph.get_node_weight(node)?.id(),
            only_in_children,
            only_in_ordering_node: only_in_ordering_node
                .into_iter()
                .map(|idx| graph.get_node_weight(idx).map(|nw| nw.id()))
                .try_collect()?,
        });
    }

    Ok(())
}

impl ValidateNode for AttributeValueNodeWeight {
    fn validate_node(
        graph: &WorkspaceSnapshotGraphVCurrent,
        issues: &mut Vec<ValidationIssue>,
        (_, attr): (&Self, NodeIndex),
    ) -> WorkspaceSnapshotGraphResult<()> {
        let Some(prop) = graph.target_opt(attr, EdgeWeightKind::Prop)? else {
            return Ok(());
        };
        let prop_kind = graph.get_node_weight(prop)?.as_prop_node_weight()?.kind();

        if prop_kind.is_container() {
            validate_child_ordering(
                graph,
                issues,
                attr,
                graph.targets(attr, EdgeWeightKindDiscriminants::Contain),
            )?;
        }
        if prop_kind == PropKind::Object {
            validate_object_children(graph, issues, attr, prop)?;
        }
        return Ok(());

        // If this is an object attribute value, check that it has child attribute values for each child prop
        // TODO check order as well
        fn validate_object_children(
            graph: &WorkspaceSnapshotGraphVCurrent,
            issues: &mut Vec<ValidationIssue>,
            attr: NodeIndex,
            prop: NodeIndex,
        ) -> WorkspaceSnapshotGraphResult<()> {
            // Check our the children we *do* have against the child props we *should* have
            let child_props: HashSet<_> = graph
                .targets(prop, EdgeWeightKindDiscriminants::Use)
                .collect();

            // Step through child avs, and record the ones we see (maybe report duplicates)
            let mut attr_content = HashMap::new();
            for child_attr in graph.targets(attr, EdgeWeightKindDiscriminants::Contain) {
                let child_attr_prop = graph.target(child_attr, EdgeWeightKind::Prop)?;
                if !child_props.contains(&child_attr_prop) {
                    issues.push(ValidationIssue::UnknownChildAttributeValue {
                        child: graph.get_node_weight(child_attr)?.id().into(),
                    });
                    continue;
                }
                let content = graph
                    .get_node_weight(child_attr)?
                    .get_attribute_value_node_weight()?
                    .value();
                if let Some(orig_content) = attr_content.insert(child_attr_prop, content) {
                    let original = graph.get_node_weight(child_attr)?.id().into();
                    let duplicate = graph.get_node_weight(child_attr)?.id().into();
                    if content == orig_content {
                        issues.push(ValidationIssue::DuplicateAttributeValue {
                            original,
                            duplicate,
                        });
                        continue;
                    } else {
                        issues.push(
                            ValidationIssue::DuplicateAttributeValueWithDifferentValues {
                                original,
                                duplicate,
                            },
                        );
                        continue;
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
                issues.push(ValidationIssue::MissingChildAttributeValues {
                    object: graph.get_node_weight(attr)?.id().into(),
                    missing_children,
                });
            }

            Ok(())
        }
    }
}

impl ValidateNode for PropNodeWeight {
    fn validate_node(
        graph: &WorkspaceSnapshotGraphVCurrent,
        issues: &mut Vec<ValidationIssue>,
        (prop_node, prop): (&Self, NodeIndex),
    ) -> WorkspaceSnapshotGraphResult<()> {
        if prop_node.kind().is_container() {
            validate_child_ordering(
                graph,
                issues,
                prop,
                graph.targets(prop, EdgeWeightKindDiscriminants::Use),
            )?;
        }

        Ok(())
    }
}

impl ValidateNode for OrderingNodeWeight {
    fn validate_node(
        graph: &WorkspaceSnapshotGraphVCurrent,
        issues: &mut Vec<ValidationIssue>,
        (ordering_node, ordering): (&Self, NodeIndex),
    ) -> WorkspaceSnapshotGraphResult<()> {
        // Collect order entries and check for duplicates
        let mut only_in_order_vec = HashSet::new();
        for &child_id in ordering_node.order() {
            if !only_in_order_vec.insert(child_id) {
                issues.push(ValidationIssue::OrderingDuplicateEntry {
                    ordering: ordering_node.id(),
                    entry: child_id,
                });
            }
        }

        // Look for extra and missing Ordinal edges that are missing from the ordering vec
        let mut only_in_ordinal_edges = Vec::new();
        for ordinal in graph.targets(ordering, EdgeWeightKindDiscriminants::Ordinal) {
            let ordinal_id = graph.get_node_weight(ordinal)?.id();
            if !only_in_order_vec.remove(&ordinal_id) {
                only_in_ordinal_edges.push(ordinal_id);
            }
        }

        // Report if there are any differences
        if !only_in_order_vec.is_empty() || !only_in_ordinal_edges.is_empty() {
            issues.push(ValidationIssue::OrderingNodeMismatch {
                ordering: ordering_node.id(),
                only_in_order_vec: only_in_order_vec.into_iter().collect(),
                only_in_ordinal_edges,
            });
        }

        Ok(())
    }
}

/// A single validation issue found in the graph
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, strum::EnumDiscriminants)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ValidationIssue {
    /// A node's children are not the same as its ordering node
    ChildOrderingMismatch {
        node: Ulid,
        only_in_children: Vec<Ulid>,
        only_in_ordering_node: Vec<Ulid>,
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
    /// Node has no ordering node
    MissingOrderingNode { node: Ulid },
    /// APA has neither a PrototypeArgumentValue or ValueSubscription edge
    MissingValue { apa: AttributePrototypeArgumentId },
    /// APA has multiple PrototypeArgumentValue and ValueSubscription edges
    MultipleValues { apa: AttributePrototypeArgumentId },
    /// Ordering node has duplicate entries in its ordering vec
    OrderingDuplicateEntry { ordering: Ulid, entry: Ulid },
    /// Ordering node has a value in its ordering node that is not in its ordering vec
    OrderingNodeMismatch {
        ordering: Ulid,
        only_in_order_vec: Vec<Ulid>,
        only_in_ordinal_edges: Vec<Ulid>,
    },
    /// A child attribute value was found under an object, but it was not associated with any
    /// of the object's child props
    UnknownChildAttributeValue { child: AttributeValueId },
}

/// Helper to format values that need extra context when being displayed (such as graphs, lookup tables,
/// etc).
pub struct WithGraph<'a, T>(pub &'a WorkspaceSnapshotGraphVCurrent, pub T);

impl std::fmt::Display for WithGraph<'_, Ulid> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, id) = self;
        match graph.get_node_weight_by_id(id) {
            Ok(NodeWeight::AttributePrototypeArgument(node)) => {
                write!(
                    f,
                    "{}",
                    WithGraph::<AttributePrototypeArgumentId>(graph, node.id().into())
                )
            }
            Ok(NodeWeight::AttributeValue(node)) => {
                write!(
                    f,
                    "{}",
                    WithGraph::<AttributeValueId>(graph, node.id().into())
                )
            }
            Ok(NodeWeight::Component(node)) => {
                write!(f, "{}", WithGraph::<ComponentId>(graph, node.id().into()))
            }
            Ok(NodeWeight::Func(node)) => {
                write!(f, "{}", WithGraph::<FuncId>(graph, node.id().into()))
            }
            Ok(NodeWeight::Prop(node)) => {
                write!(f, "{}", WithGraph::<PropId>(graph, node.id().into()))
            }
            Ok(node) => write!(f, "node {}", node.id()),
            Err(err) => write!(f, "node <error: {err}> ({id})>"),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, AttributeValueId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, id) = self;
        match av_path_from_root(graph, id) {
            Ok((_, path)) if path.is_root() => write!(f, "root av ({id})"),
            Ok((_, path)) => write!(f, "av {path} ({id})"),
            Err(err) => write!(f, "av <error: {err}> ({id})"),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, AttributePrototypeArgumentId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(_, id) = self;
        write!(f, "apa {id}")
    }
}

impl std::fmt::Display for WithGraph<'_, ComponentId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(_, id) = self;
        write!(f, "component {id}")
    }
}

impl std::fmt::Display for WithGraph<'_, FuncId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, id) = self;
        match graph
            .get_node_weight_by_id(id)
            .and_then(|node| node.get_func_node_weight().map_err(Into::into))
        {
            Ok(node) => write!(f, "func {} ({id})", node.name()),
            Err(err) => write!(f, "func <error: {err}> ({id})"),
        }
    }
}

impl std::fmt::Display for WithGraph<'_, PropId> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let &WithGraph(graph, id) = self;
        match prop_path_from_root(graph, id) {
            Ok((_, path)) if path.is_root() => write!(f, "root prop ({id})"),
            Ok((_, path)) => write!(f, "prop {path} ({id})"),
            Err(err) => write!(f, "prop <error: {err}> ({id})"),
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
            ValidationIssue::ChildOrderingMismatch {
                node,
                ref only_in_children,
                ref only_in_ordering_node,
            } => {
                write!(f, "Child ordering mismatch for {}.", WithGraph(graph, node))?;
                if !only_in_children.is_empty() {
                    write!(
                        f,
                        " - only in children: {}",
                        only_in_children
                            .iter()
                            .map(|&id| format!("{}", WithGraph(graph, id)))
                            .join(", ")
                    )?;
                }
                if !only_in_ordering_node.is_empty() {
                    write!(
                        f,
                        " - only in ordering node: {}",
                        only_in_ordering_node
                            .iter()
                            .map(|&id| format!("{}", WithGraph(graph, id)))
                            .join(", ")
                    )?;
                }
                Ok(())
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
                    "object {} missing child attribute values for props: {}",
                    WithGraph(graph, object),
                    missing_children
                        .iter()
                        .map(|&child_prop| format!("{}", WithGraph(graph, child_prop)))
                        .join(", ")
                )
            }
            ValidationIssue::MissingOrderingNode { node } => {
                write!(f, "{} has no ordering node", WithGraph(graph, node))
            }
            ValidationIssue::MissingValue { apa } => {
                write!(f, "{} has no value", WithGraph(graph, apa))
            }
            ValidationIssue::MultipleValues { apa } => {
                write!(f, "{} has multiple values", WithGraph(graph, apa))
            }
            ValidationIssue::OrderingDuplicateEntry { ordering, entry } => {
                write!(
                    f,
                    "Ordering {} has duplicate entry {}",
                    WithGraph(graph, ordering),
                    WithGraph(graph, entry)
                )
            }
            ValidationIssue::OrderingNodeMismatch {
                ordering,
                ref only_in_order_vec,
                ref only_in_ordinal_edges,
            } => {
                write!(
                    f,
                    "Ordering {} has mismatched entries.",
                    WithGraph(graph, ordering)
                )?;
                if !only_in_order_vec.is_empty() {
                    write!(
                        f,
                        " - only in order vec: {}",
                        only_in_order_vec
                            .iter()
                            .map(|&id| format!("{}", WithGraph(graph, id)))
                            .join(", ")
                    )?;
                }
                if !only_in_ordinal_edges.is_empty() {
                    write!(
                        f,
                        " - only in ordinal edges: {}",
                        only_in_ordinal_edges
                            .iter()
                            .map(|&id| format!("{}", WithGraph(graph, id)))
                            .join(", ")
                    )?;
                }
                Ok(())
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

        // ECS Load Balancer Configuration
        "containerNameToLBConfigContainerName",
        "containerPortToLBConfigContainerPort",

        // ECS Container Definition Port Mapping
        "containerPortToOutputSocket",
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

        // Subnet
        "si:awsSubnetIdFromResource",
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
    let mut path = jsonptr::PointerBuf::root();
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
    let mut path = jsonptr::PointerBuf::root();

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
