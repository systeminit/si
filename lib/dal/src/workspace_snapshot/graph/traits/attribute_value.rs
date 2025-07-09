use std::collections::HashMap;

use si_id::{
    AttributePrototypeId,
    AttributeValueId,
    PropId,
};

use crate::{
    PropKind,
    attribute::value::AttributeValueResult,
};

pub trait AttributeValueExt {
    fn attribute_value_tree(
        &self,
        root_attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<AttributeValueTree>;

    fn child_attribute_values(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValueId>>;

    fn child_attribute_values_in_order(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Vec<AttributeValueId>>;

    fn component_prototype_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributePrototypeId>>;

    fn key_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<String>>;

    fn prop_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> AttributeValueResult<Option<PropId>>;
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct AttributeValueTreeEntry {
    pub attribute_value_id: AttributeValueId,
    pub prop_id: PropId,
    pub prop_kind: PropKind,
}

/// The tree of [`AttributeValueId`] starting from the selected root [`AttributeValueId`].
#[derive(Debug)]
pub struct AttributeValueTree {
    root_av_entry: AttributeValueTreeEntry,
    children: HashMap<AttributeValueId, Vec<AttributeValueTreeEntry>>,
}

impl AttributeValueTree {
    /// Construct a new empty tree with the given root [`AttributeValueId`].
    pub fn new(
        root_av_id: AttributeValueId,
        root_av_prop_id: PropId,
        root_av_prop_kind: PropKind,
    ) -> Self {
        let mut children = HashMap::new();
        children.insert(root_av_id, Vec::new());
        let root_av_entry = AttributeValueTreeEntry {
            attribute_value_id: root_av_id,
            prop_id: root_av_prop_id,
            prop_kind: root_av_prop_kind,
        };

        Self {
            root_av_entry,
            children,
        }
    }

    /// Add a child [`AttributeValueTreeEntry`] under the given parent [`AttributeValueId`].
    pub fn add_child(
        &mut self,
        parent_av_id: AttributeValueId,
        child_av_entry: AttributeValueTreeEntry,
    ) {
        self.children
            .entry(parent_av_id)
            .or_default()
            .push(child_av_entry);
    }

    /// Add multiple children [`AttributeValueTreeEntry`] under the given parent [`AttributeValueId`].
    pub fn add_children(
        &mut self,
        parent_av_id: AttributeValueId,
        mut new_children: Vec<AttributeValueTreeEntry>,
    ) {
        self.children
            .entry(parent_av_id)
            .and_modify(|children| children.append(&mut new_children))
            .or_insert(new_children);
    }

    /// Get the children [`AttributeValueTreeEntry`] of the given parent [`AttributeValueId`].
    pub fn children_of(&self, av_id: AttributeValueId) -> &[AttributeValueTreeEntry] {
        self.children
            .get(&av_id)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// The total number of [`AttributeValue`] in the tree, including the root.
    pub fn count(&self) -> usize {
        self.children.len()
    }

    /// Get the root [`AttributeValueTreeEntry`].
    pub fn root(&self) -> AttributeValueTreeEntry {
        self.root_av_entry
    }
}
