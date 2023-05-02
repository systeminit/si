use object_tree::{Hash, HashedNode};
use petgraph::prelude::*;

use super::{PkgResult, SiPkgError, Source};

use crate::{node::PkgNode, ActionSpec, ActionSpecKind, WorkflowSpec};

#[derive(Clone, Debug)]
pub struct SiPkgWorkflow<'a> {
    func_unique_id: Hash,
    title: String,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgWorkflow<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::Workflow(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::WORKFLOW_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let workflow = Self {
            func_unique_id: node.func_unique_id,
            title: node.title,
            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        };

        Ok(workflow)
    }

    pub fn func_unique_id(&self) -> Hash {
        self.func_unique_id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn actions(&self) -> PkgResult<Vec<SiPkgAction>> {
        let mut actions = vec![];
        for idx in self
            .source
            .graph
            .neighbors_directed(self.source.node_idx, Outgoing)
        {
            actions.push(SiPkgAction::from_graph(self.source.graph, idx)?);
        }

        Ok(actions)
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> TryFrom<SiPkgWorkflow<'a>> for WorkflowSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgWorkflow<'a>) -> Result<Self, Self::Error> {
        let mut builder = WorkflowSpec::builder();

        builder
            .title(value.title())
            .func_unique_id(value.func_unique_id);

        for action in value.actions()? {
            builder.action(action.try_into()?);
        }

        Ok(builder.build()?)
    }
}

#[derive(Clone, Debug)]
pub struct SiPkgAction<'a> {
    kind: ActionSpecKind,
    name: String,

    hash: Hash,
    source: Source<'a>,
}

impl<'a> SiPkgAction<'a> {
    pub fn from_graph(
        graph: &'a Graph<HashedNode<PkgNode>, ()>,
        node_idx: NodeIndex,
    ) -> PkgResult<Self> {
        let hashed_node = &graph[node_idx];
        let node = match hashed_node.inner() {
            PkgNode::Action(node) => node.clone(),
            unexpected => {
                return Err(SiPkgError::UnexpectedPkgNodeType(
                    PkgNode::ACTION_KIND_STR,
                    unexpected.node_kind_str(),
                ))
            }
        };

        let action = Self {
            kind: node.kind,
            name: node.name,
            hash: hashed_node.hash(),
            source: Source::new(graph, node_idx),
        };

        Ok(action)
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> ActionSpecKind {
        self.kind
    }

    pub fn hash(&self) -> Hash {
        self.hash
    }

    pub fn source(&self) -> &Source<'a> {
        &self.source
    }
}

impl<'a> TryFrom<SiPkgAction<'a>> for ActionSpec {
    type Error = SiPkgError;

    fn try_from(value: SiPkgAction<'a>) -> Result<Self, Self::Error> {
        Ok(ActionSpec::builder()
            .kind(value.kind)
            .name(value.name)
            .build()?)
    }
}
