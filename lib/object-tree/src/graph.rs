//! An implementation of a tree structure that can be transformed into a Merkle n-ary acyclic
//! directed graph (i.e. "DAG").

use std::{
    collections::HashMap,
    fmt,
    io::{self, BufRead, Cursor, Write},
    str::FromStr,
};

use petgraph::prelude::*;
use serde::Serialize;
use strum::{AsRefStr, EnumString};
use thiserror::Error;

use crate::Hash;

const KEY_VERSION_STR: &str = "version";
const KEY_NODE_KIND_STR: &str = "node_kind";

const VAL_VERSION_STR: &str = "1";

/// The canonical serialized form of a new line.
pub const NL: char = '\n';

/// An error that can be returned when working with tree and graph types.
#[remain::sorted]
#[derive(Debug, Error)]
pub enum GraphError {
    /// When an attempt to get a slice from the BufRead internal buffer fails
    #[error("could not get an expected range from the BufRead internal buffer")]
    BufReadRangeError,
    /// When a checked arithmetic operation returns [`None`]
    #[error("checked arithmetic failed: {0}")]
    CheckedArithmeticFailure(&'static str),
    /// When parsing a serialized node representation and a valid version was found
    #[error("invalid node version when parsing from bytes: {0}")]
    InvalidNodeVersion(String),
    /// When an error is returned while reading serialized node representation
    #[error("error reading node representation from bytes")]
    IoRead(#[source] io::Error),
    /// When an error is returned while writing a serialized node representation
    #[error("error writing node representation as bytes")]
    IoWrite(#[source] io::Error),
    /// When a root node was not found after traversing a tree
    #[error("root node not set after traversing tree")]
    MissingRootNode,
    /// When multiple root nodes were found while traversing a tree
    #[error("root node already set, cannot have multiple roots in tree")]
    MultipleRootNode,
    /// When a node weight is not found for a given index
    #[error("node weight not found for index ({0}): {1}")]
    NodeWeightNotFound(usize, &'static str),
    /// When parsing a serialized node from bytes returns an error
    #[error("error parsing node from bytes: {0}")]
    Parse(#[source] Box<dyn std::error::Error + Send + Sync + 'static>),
    /// When parsing a serialized node from bytes and an invalid state is found
    #[error("error parsing node from bytes: {0}")]
    ParseCustom(String),
    /// When a blank line was expected while parsing a serialized node
    #[error("parsing line was expected to be blank, but got '{0}'")]
    ParseLineBlank(String),
    /// When a line was expected to contain a given key while parsing a serialized node
    #[error("parsing key/value line error, expected key '{0}', but got '{1}'")]
    ParseLineExpectedKey(String, String),
    /// When a line failed to parse as a key/value line while parsing a serialize node
    #[error("could not parse line as 'key=value': '{0}'")]
    ParseLineKeyValueFormat(String),
    /// When a child node is missing a hash value while computing a hashing tree
    #[error("unhashed child node for '{0}' with name: {1}")]
    UnhashedChild(String, String),
    /// When a node is missing a hash value while computing a hashing tree
    #[error("unhashed node with name: {0}")]
    UnhashedNode(String),
    /// When a hash value failed to verify an expected value
    #[error("failed to verify hash; expected={0}, computed={1}")]
    Verify(Hash, Hash),
}

impl GraphError {
    /// Returns a parsing error which wraps the given inner error.
    pub fn parse<E>(err: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Parse(Box::new(err))
    }

    /// Return a custom parsing error which contains the given message.
    pub fn parse_custom(msg: impl Into<String>) -> Self {
        Self::ParseCustom(msg.into())
    }
}

/// Trait for types that can serialize to a representation of bytes.
pub trait WriteBytes {
    /// Writes a serialized version of `self` to the writer as bytes.
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError>;

    /// Builds and returns a `Vec` of bytes which is a serialized representation of `self`.
    fn to_bytes(&self) -> Result<Vec<u8>, GraphError> {
        let mut writer = Cursor::new(Vec::new());
        self.write_bytes(&mut writer)?;
        Ok(writer.into_inner())
    }
}

/// Trait for types which can compute and verify their own [`struct@Hash`] value.
pub trait VerifyHash: WriteBytes {
    /// Returns a pre-computed [`struct@Hash`] value for `self`.
    fn hash(&self) -> &Hash;

    /// Recomputes a [`struct@Hash`] value for `self` and confirms it matches the pre-computed Hash
    /// value.
    fn verify_hash(&self) -> Result<(), GraphError> {
        let input = self.to_bytes()?;
        let computed = Hash::new(&input);

        if self.hash() == &computed {
            Ok(())
        } else {
            Err(GraphError::Verify(*self.hash(), computed))
        }
    }
}

/// Trait for types that can deserialize its representation from bytes.
pub trait ReadBytes {
    /// Reads a serialized version of `self` from a reader over bytes.
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized;

    /// Builds and returns a new instance which was deserialized from a `Vec` of bytes.
    fn from_bytes(buf: Vec<u8>) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let mut reader = Cursor::new(buf);
        Self::read_bytes(&mut reader)
    }
}

/// Trait for types that return a String representation of its name.
pub trait NameStr {
    /// Returns a name as a `&str`.
    fn name(&self) -> &str;
}

/// Whether a `Node` (or a node-related type) is a leaf or a tree.
///
/// A *leaf* is a node which contains no children and a *tree* is a node which contains children.
#[remain::sorted]
#[derive(AsRefStr, Debug, Clone, Copy, EnumString, Eq, Hash, PartialEq, Serialize)]
#[strum(serialize_all = "camelCase")]
pub enum NodeKind {
    /// A leaf node has no children.
    Leaf,
    /// A tree node has children.
    Tree,
}

/// A node entry is a representation of a child node in a parent node's serialized representation.
#[derive(Clone, Debug)]
pub(crate) struct NodeEntry {
    kind: NodeKind,
    hash: Hash,
    name: String,
}

impl NodeEntry {
    pub(crate) fn new(kind: NodeKind, hash: Hash, name: impl Into<String>) -> Self {
        Self {
            kind,
            hash,
            name: name.into(),
        }
    }

    #[must_use]
    pub(crate) fn hash(&self) -> Hash {
        self.hash
    }
}

impl WriteBytes for NodeEntry {
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write!(
            writer,
            "{} {} {}{NL}",
            self.kind.as_ref(),
            self.hash,
            self.name
        )
        .map_err(GraphError::IoWrite)
    }
}

/// An un-hashed node in a tree.
#[derive(Clone, Debug)]
struct Node<T> {
    kind: NodeKind,
    inner: T,
}

/// FIXME(fnichol): document
pub trait NodeChild {
    /// The type of `Node` which the children can resolve into.
    type NodeType;

    /// FIXME(fnichol): document
    fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType>;
}

/// An un-hashed tree node which includes its children.
pub struct NodeWithChildren<T> {
    kind: NodeKind,
    inner: T,
    children: Vec<Box<dyn NodeChild<NodeType = T>>>,
}

impl<T> NodeWithChildren<T> {
    /// Creates a new instance given a kind, an inner type `T` and its children.
    pub fn new(kind: NodeKind, inner: T, children: Vec<Box<dyn NodeChild<NodeType = T>>>) -> Self {
        Self {
            kind,
            inner,
            children,
        }
    }

    fn into_parts(self) -> (Node<T>, Vec<Box<NodeWithChildren<T>>>) {
        let node = Node {
            kind: self.kind,
            inner: self.inner,
        };
        let children = self
            .children
            .into_iter()
            .map(|child| Box::new(child.as_node_with_children()))
            .collect();

        (node, children)
    }
}

/// A reference to an un-hashed node which includes a slice of [`NodeEntry`] items representing its
/// children, if any.
struct NodeWithEntriesRef<'a, T> {
    kind: NodeKind,
    inner: &'a T,
    entries: &'a [NodeEntry],
}

impl<'a, T> NodeWithEntriesRef<'a, T> {
    /// Creates a new instance given a kind, an innter type `T` and a slice of [`NodeEntry`] items
    /// representing its children, if any.
    fn new(kind: NodeKind, inner: &'a T, entries: &'a [NodeEntry]) -> Self {
        Self {
            kind,
            inner,
            entries,
        }
    }
}

impl<'a, T> WriteBytes for NodeWithEntriesRef<'a, T>
where
    T: WriteBytes,
{
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        write_header_bytes(writer, self.kind)?;

        write_separator_bytes(writer)?;

        self.inner.write_bytes(writer)?;

        if !self.entries.is_empty() {
            write_separator_bytes(writer)?;

            // all entries must be deterministically ordered, and that is by entry name sorted
            // lexically
            let mut sorted_entries: Vec<_> = self.entries.iter().collect();
            sorted_entries.sort_by_key(|k| &k.name);

            for entry in sorted_entries {
                entry.write_bytes(writer)?;
            }
        }

        Ok(())
    }
}

/// An un-hashed node which includes a `Vec` of [`NodeEntry`] items representing its children, if
/// any.
pub(crate) struct NodeWithEntries<T> {
    kind: NodeKind,
    inner: T,
    entries: Vec<NodeEntry>,
}

impl<T> ReadBytes for NodeWithEntries<T>
where
    T: ReadBytes,
{
    fn read_bytes<R: BufRead>(reader: &mut R) -> Result<Option<Self>, GraphError>
    where
        Self: std::marker::Sized,
    {
        let version_str = read_key_value_line(reader, KEY_VERSION_STR)?;
        if version_str != VAL_VERSION_STR {
            return Err(GraphError::InvalidNodeVersion(version_str));
        }

        let kind_str = read_key_value_line(reader, KEY_NODE_KIND_STR)?;
        let kind = NodeKind::from_str(&kind_str).map_err(GraphError::parse)?;

        read_empty_line(reader)?;

        Ok(match T::read_bytes(reader)? {
            Some(node) => {
                let entries = match kind {
                    NodeKind::Leaf => vec![],
                    NodeKind::Tree => {
                        read_empty_line(reader)?;

                        read_node_entry_lines(reader)?
                    }
                };

                Some(Self {
                    kind,
                    inner: node,
                    entries,
                })
            }
            None => None,
        })
    }
}

/// A tree structure that is used to compute a fully hashed Merkle DAG.
#[derive(Clone, Debug)]
struct HashingTree<T> {
    graph: Graph<Node<T>, ()>,
    root_idx: NodeIndex,
    hashes: HashMap<NodeIndex, Hash>,
}

impl<T> HashingTree<T> {
    /// Builds a new [`HashingTree`] from a root [`NodeWithChildren`] that can be hashed and
    /// computed.
    ///
    /// # Errors
    ///
    /// Return `Err` if multiple root nodes are found (which is invalid for a tree) or if no root
    /// nodes are found once the tree is fully processed (which is also invalid for a tree).
    fn create_from_root(node: NodeWithChildren<T>) -> Result<HashingTree<T>, GraphError> {
        let mut graph = Graph::new();
        let mut root_idx: Option<NodeIndex> = None;
        let hashes = HashMap::new();

        let mut stack: Vec<(_, Option<NodeIndex>)> = vec![(node, None)];

        while let Some((node_with_children, parent_idx)) = stack.pop() {
            let (node, children) = node_with_children.into_parts();

            let node_idx = graph.add_node(node);

            match parent_idx {
                Some(parent_idx) => {
                    graph.add_edge(parent_idx, node_idx, ());
                }
                None => match root_idx {
                    None => {
                        root_idx = Some(node_idx);
                    }
                    Some(_) => return Err(GraphError::MultipleRootNode),
                },
            };

            for child_node_with_children in children.into_iter().rev() {
                stack.push((*child_node_with_children, Some(node_idx)));
            }
        }

        match root_idx {
            Some(root_idx) => Ok(HashingTree {
                graph,
                root_idx,
                hashes,
            }),
            None => Err(GraphError::MissingRootNode),
        }
    }

    /// Builds a new [`ObjectTree`] by computing hashes for all nodes.
    ///
    /// # Errors
    ///
    /// Return `Err` if:
    ///
    /// - An un-hashed child node is found during depth-first post-order tree traversal (i.e. this
    /// implies all children have not yet been computed which is invalid)
    /// - An I/O error occurs when serializing node representations to bytes
    fn hash_tree(mut self) -> Result<ObjectTree<T>, GraphError>
    where
        T: Clone + NameStr + WriteBytes,
    {
        self.compute_hashes()?;
        self.create_hashed_tree()
    }

    fn compute_hashes(&mut self) -> Result<(), GraphError>
    where
        T: NameStr + WriteBytes,
    {
        let mut dfspo = DfsPostOrder::new(&self.graph, self.root_idx);

        while let Some(node_idx) = dfspo.next(&self.graph) {
            let node = self
                .graph
                .node_weight(node_idx)
                .ok_or(GraphError::NodeWeightNotFound(
                    node_idx.index(),
                    "could not find node for next item in dfspo",
                ))?;

            // Create an entry for each direct child
            let mut entries = Vec::new();
            for child_idx in self.graph.neighbors_directed(node_idx, Outgoing) {
                let child_node =
                    self.graph
                        .node_weight(child_idx)
                        .ok_or(GraphError::NodeWeightNotFound(
                            child_idx.index(),
                            "could not find child weight for index",
                        ))?;
                let child_hash = self.hashes.get(&child_idx).ok_or_else(|| {
                    GraphError::UnhashedChild(
                        node.inner.name().to_string(),
                        child_node.inner.name().to_string(),
                    )
                })?;

                entries.push(NodeEntry {
                    kind: child_node.kind,
                    hash: *child_hash,
                    name: child_node.inner.name().to_string(),
                });
            }

            // Serialize node to bytes and compute hash
            let mut writer = Cursor::new(Vec::new());
            NodeWithEntriesRef::new(node.kind, &node.inner, &entries).write_bytes(&mut writer)?;
            let computed_hash = Hash::new(&writer.into_inner());

            self.hashes.insert(node_idx, computed_hash);
        }

        Ok(())
    }

    fn create_hashed_tree(self) -> Result<ObjectTree<T>, GraphError>
    where
        T: Clone + NameStr,
    {
        #[derive(Debug)]
        struct StackEntry<T> {
            hashed_node: HashedNode<T>,
            other_idx: NodeIndex,
            parent_idx: Option<NodeIndex>,
        }

        let other_root_node = self
            .graph
            .node_weight(self.root_idx)
            .ok_or(GraphError::NodeWeightNotFound(
                self.root_idx.index(),
                "could not find weight for other root node",
            ))?
            .clone();
        let other_root_node_hash = self
            .hashes
            .get(&self.root_idx)
            .ok_or_else(|| GraphError::UnhashedNode(other_root_node.inner.name().to_string()))?;

        let mut graph = Graph::new();
        let mut root_idx: Option<NodeIndex> = None;

        let mut stack = vec![StackEntry {
            hashed_node: HashedNode::new(other_root_node, *other_root_node_hash),
            other_idx: self.root_idx,
            parent_idx: None,
        }];

        while let Some(entry) = stack.pop() {
            let node_idx = graph.add_node(entry.hashed_node);

            match entry.parent_idx {
                Some(parent_idx) => {
                    graph.add_edge(parent_idx, node_idx, ());
                }
                None => match root_idx {
                    None => {
                        root_idx = Some(node_idx);
                    }
                    Some(_) => return Err(GraphError::MultipleRootNode),
                },
            };

            for other_child_idx in self.graph.neighbors_directed(entry.other_idx, Outgoing) {
                let other_node = self
                    .graph
                    .node_weight(other_child_idx)
                    .ok_or(GraphError::NodeWeightNotFound(
                        other_child_idx.index(),
                        "could not find other child node for index",
                    ))?
                    .clone();
                let other_node_hash = self
                    .hashes
                    .get(&other_child_idx)
                    .ok_or_else(|| GraphError::UnhashedNode(other_node.inner.name().to_string()))?;

                stack.push(StackEntry {
                    hashed_node: HashedNode::new(other_node, *other_node_hash),
                    other_idx: other_child_idx,
                    parent_idx: Some(node_idx),
                });
            }
        }

        match root_idx {
            Some(root_idx) => Ok(ObjectTree { graph, root_idx }),
            None => Err(GraphError::MissingRootNode),
        }
    }
}

/// A tree of hashed nodes of type `T`.
///
/// The tree can be considered a Merkle DAG (directed acyclic graph) or a Merkle n-ary tree (that
/// is not a binary or "balanced" tree). A node is hashed over its serialized bytes representation
/// which includes the hashes of all of its children. In this way it is possible to determine if 2
/// nodes are equivalent in that they both represent identical sub-trees and can be mathematically
/// verified.
#[derive(Clone, Debug)]
pub struct ObjectTree<T> {
    graph: Graph<HashedNode<T>, ()>,
    root_idx: NodeIndex,
}

impl<T> ObjectTree<T> {
    /// Creates an `ObjectTree` from an un-hashed root node of type `T` with its children.
    pub fn create_from_root(node: NodeWithChildren<T>) -> Result<Self, GraphError>
    where
        T: Clone + NameStr + WriteBytes,
    {
        HashingTree::create_from_root(node)?.hash_tree()
    }

    /// Returns the tree as a [`Graph`] of [`HashedNode`] items and a pointer to the root node.
    pub fn as_petgraph(&self) -> (&Graph<HashedNode<T>, ()>, NodeIndex) {
        (&self.graph, self.root_idx)
    }

    /// Builds a new `ObjectTree` from an exisiting [`Graph`] of [`HashedNode`] items and a root
    /// index pointer.
    #[must_use]
    pub(crate) fn new(graph: Graph<HashedNode<T>, ()>, root_idx: NodeIndex) -> Self {
        Self { graph, root_idx }
    }
}

/// A hashed node of type `T`.
#[derive(Clone, Eq, PartialEq, Serialize)]
pub struct HashedNode<T> {
    kind: NodeKind,
    hash: Hash,
    inner: T,
}

impl<T> fmt::Debug for HashedNode<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HashedNode")
            .field("kind", &self.kind)
            // This is pragmatic--the full hashes can lead to very long output lines and visual Dot
            // graph images
            .field("hash", &self.hash.short_string())
            .field("inner", &self.inner)
            .finish()
    }
}

impl<T> HashedNode<T> {
    fn new(node: Node<T>, hash: Hash) -> Self {
        Self {
            kind: node.kind,
            hash,
            inner: node.inner,
        }
    }

    /// Returns the [`NodeKind`] of this node.
    pub fn kind(&self) -> NodeKind {
        self.kind
    }

    /// Returns the pre-computed [`struct@Hash`] of this node.
    pub fn hash(&self) -> Hash {
        self.hash
    }

    /// Returns the inner representation `T` of this node.
    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T> NameStr for HashedNode<T>
where
    T: NameStr,
{
    fn name(&self) -> &str {
        self.inner.name()
    }
}

/// A hashed node which includes a `Vec` of [`NodeEntry`] items representing its children, if any.
pub(crate) struct HashedNodeWithEntries<T> {
    kind: NodeKind,
    hash: Hash,
    inner: T,
    entries: Vec<NodeEntry>,
}

impl<T> HashedNodeWithEntries<T> {
    pub(crate) fn new(hashed_node: HashedNode<T>, entries: Vec<NodeEntry>) -> Self {
        Self {
            kind: hashed_node.kind,
            hash: hashed_node.hash,
            inner: hashed_node.inner,
            entries,
        }
    }

    pub(crate) fn from_node_with_entries_and_hash(
        node_with_entries: NodeWithEntries<T>,
        hash: Hash,
    ) -> Self {
        Self {
            kind: node_with_entries.kind,
            hash,
            inner: node_with_entries.inner,
            entries: node_with_entries.entries,
        }
    }

    pub(crate) fn hash(&self) -> Hash {
        self.hash
    }

    fn as_node_with_entries_ref(&self) -> NodeWithEntriesRef<'_, T> {
        NodeWithEntriesRef {
            kind: self.kind,
            inner: &self.inner,
            entries: &self.entries,
        }
    }
}

impl<T> WriteBytes for HashedNodeWithEntries<T>
where
    T: WriteBytes,
{
    fn write_bytes<W: Write>(&self, writer: &mut W) -> Result<(), GraphError> {
        self.as_node_with_entries_ref().write_bytes(writer)
    }
}

impl<T> VerifyHash for HashedNodeWithEntries<T>
where
    T: WriteBytes,
{
    fn hash(&self) -> &Hash {
        &self.hash
    }
}

impl<T> From<HashedNodeWithEntries<T>> for (HashedNode<T>, Vec<NodeEntry>) {
    fn from(value: HashedNodeWithEntries<T>) -> Self {
        (
            HashedNode {
                kind: value.kind,
                hash: value.hash,
                inner: value.inner,
            },
            value.entries,
        )
    }
}

impl<T> From<HashedNode<T>> for NodeEntry
where
    T: NameStr,
{
    fn from(value: HashedNode<T>) -> Self {
        Self {
            kind: value.kind,
            hash: value.hash,
            name: value.inner.name().to_string(),
        }
    }
}

fn parse_key_line(line: &str) -> Result<(&str, usize, &str), GraphError> {
    let (line_key_and_len, line_value) = match line.split_once('=') {
        Some((key_and_len, value)) => (key_and_len, value),
        None => return Err(GraphError::ParseLineKeyValueFormat(line.into())),
    };

    let (line_key, len) = match line_key_and_len.split_once(':') {
        Some((key, len_str)) => (key, usize::from_str(len_str).map_err(GraphError::parse)?),
        None => return Err(GraphError::ParseLineKeyValueFormat(line.into())),
    };

    Ok((line_key, len, line_value))
}

/// Attempts to read a key/value formatted line for the given key, returning the value as a
/// `Option<String>`.
///
/// If the key does not match (or if the file is at EOF, or if the key is larger than the BufRead
/// internal buffer of 8Kb), returns None, but does not consume the buffer, allowing us to try and
/// read a different key value line.
///
/// Used for reading key value pairs that may or may not be present in an object.
///
/// # Errors
///
/// Returns an `Err` if
///
/// - An I/O error occurs while reading from the reader
/// - If the line does not parse as a key/value line
/// - If the key name in the parsed line does not match the expected key name
pub fn read_key_value_line_opt<R: BufRead>(
    reader: &mut R,
    key: impl AsRef<str>,
) -> Result<Option<String>, GraphError> {
    const ASCII_COLON: u8 = 58;

    let key_bytes = key.as_ref().as_bytes();
    let current_buf = reader.fill_buf().map_err(GraphError::IoRead)?;
    if current_buf.len() < key_bytes.len().saturating_add(1) {
        return Ok(None);
    }

    let current_buf_key = current_buf
        .get(..key_bytes.len())
        .ok_or(GraphError::BufReadRangeError)?;

    match std::str::from_utf8(current_buf_key) {
        // If this fails its because we cut the utf8 off short in the middle of a multibyte run.
        // If that's the case, our key does not match.
        Err(_) => Ok(None),
        Ok(utf8_buf_key) => {
            if utf8_buf_key == key.as_ref()
                && current_buf
                    .get(key_bytes.len())
                    .ok_or(GraphError::BufReadRangeError)?
                    == &ASCII_COLON
            {
                Ok(Some(read_key_value_line(reader, key)?))
            } else {
                Ok(None)
            }
        }
    }
}

/// Reads a key/value formatted line from a reader and returns the value as a `String`.
///
/// # Errors
///
/// Returns an `Err` if:
///
/// - An I/O error occurs while reading from the reader
/// - If the line does not parse as a key/value line
/// - If the key name in the parsed line does not match the expected key name
pub fn read_key_value_line<R: BufRead>(
    reader: &mut R,
    key: impl AsRef<str>,
) -> Result<String, GraphError> {
    let mut line = String::new();
    reader.read_line(&mut line).map_err(GraphError::IoRead)?;

    let (line_key, len, line_value) = parse_key_line(&line)?;

    if line_key != key.as_ref() {
        return Err(GraphError::ParseLineExpectedKey(
            key.as_ref().to_string(),
            line_key.to_string(),
        ));
    }

    if line_value.len().saturating_sub(1) == len {
        let mut chars = line_value.chars();
        chars.next_back();
        Ok(chars.as_str().to_owned())
    } else if len < line_value.len() {
        Err(GraphError::ParseCustom(format!(
            "Expected at most {len} characters, got {} characters in {line_value}",
            line_value.len()
        )))
    } else {
        // Safe remaining bytes operation.
        let remaining_bytes =
            len.checked_sub(line_value.len())
                .ok_or(GraphError::CheckedArithmeticFailure(
                    "could not compute remaining bytes",
                ))?;

        // Safe length operation.
        let length = remaining_bytes
            .checked_add(1)
            .ok_or(GraphError::CheckedArithmeticFailure(
                "could not compute length for remaining vec",
            ))?;

        let mut remaining = vec![0; length];
        reader
            .read_exact(&mut remaining)
            .map_err(GraphError::IoRead)?;

        let mut remaining = std::str::from_utf8(&remaining)
            .map_err(GraphError::parse)?
            .chars();
        remaining.next_back();

        let value = format!("{}{}", line_value, remaining.as_str());

        Ok(value)
    }
}

/// Reads an empty line from a reader.
///
/// # Errors
///
/// Returns an `Err` if:
///
/// - An I/O error occurs while reading from the reader
/// - If the line is not empty as expected
fn read_empty_line<R: BufRead>(reader: &mut R) -> Result<(), GraphError> {
    let mut line = String::with_capacity(0);
    reader.read_line(&mut line).map_err(GraphError::IoRead)?;

    if line.trim_end().is_empty() {
        Ok(())
    } else {
        Err(GraphError::ParseLineBlank(line))
    }
}

/// Reads, parses, and return a `Vec` of [`NodeEntry`] items from a reader.
///
/// # Errors
///
/// Returns an `Err` if:
///
/// - An I/O error occurs while reading from the reader
/// - If the line can't be parsed as a node entry line
/// - If the node kind can't be parsed from the line
/// - If the hash value can't be parsed from the line
/// - If the name can't be parsed from the line
fn read_node_entry_lines<R: BufRead>(reader: &mut R) -> Result<Vec<NodeEntry>, GraphError> {
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line.map_err(GraphError::IoRead)?;
        let mut parts: Vec<_> = line.splitn(3, ' ').collect();

        let name = match parts.pop() {
            Some(s) => s.to_string(),
            None => return Err(GraphError::parse_custom("missing name field in entry line")),
        };
        let hash = match parts.pop() {
            Some(s) => Hash::from_str(s).map_err(GraphError::parse)?,
            None => return Err(GraphError::parse_custom("missing hash field in entry line")),
        };
        let kind = match parts.pop() {
            Some(s) => NodeKind::from_str(s).map_err(GraphError::parse)?,
            None => return Err(GraphError::parse_custom("missing kind field in entry line")),
        };

        if !parts.is_empty() {
            return Err(GraphError::parse_custom(format!(
                "entry line has more than 3 fields: {}",
                line
            )));
        }

        entries.push(NodeEntry { kind, hash, name });
    }

    Ok(entries)
}

/// Writes a node header to a writer.
///
/// # Errors
///
/// Returns `Err` if an I/O error occurs while writing to the writer
fn write_header_bytes<W: Write>(writer: &mut W, kind: NodeKind) -> Result<(), GraphError> {
    write_key_value_line(writer, KEY_VERSION_STR, VAL_VERSION_STR)?;
    write_key_value_line(writer, KEY_NODE_KIND_STR, kind.as_ref())?;
    Ok(())
}

/// Writes a key/value formatted line to a writer with the given key and value.
///
/// # Errors
///
/// Returns `Err` if an I/O error occurs while writing to the writer
pub fn write_key_value_line<W: Write>(
    writer: &mut W,
    key: impl fmt::Display,
    value: impl fmt::Display,
) -> Result<(), GraphError> {
    let value: String = value.to_string();
    let len = value.len();
    write!(writer, "{key}:{len}={value}{NL}").map_err(GraphError::IoWrite)
}

/// Writes a separator/blank line to a writer.
///
/// # Errors
///
/// Returns `Err` if an I/O error occurs while writing to the writer
fn write_separator_bytes<W: Write>(writer: &mut W) -> Result<(), GraphError> {
    write!(writer, "{NL}").map_err(GraphError::IoWrite)
}

// mod ahhhh {
//     use super::NodeKind;
//
//     #[derive(Debug)]
//     struct Node<T> {
//         kind: NodeKind,
//         inner: T,
//     }
//
//     trait Child {
//         type NodeType;
//
//         fn as_node_with_children(&self) -> NodeWithChildren<Self::NodeType>;
//     }
//
//     struct NodeWithChildren<T> {
//         node: Node<T>,
//         children: Vec<Box<dyn Child<NodeType = T>>>,
//     }
//
//     impl<T> NodeWithChildren<T> {
//         fn new(node: Node<T>, children: Vec<Box<dyn Child<NodeType = T>>>) -> Self {
//             Self { node, children }
//         }
//
//         fn into_parts(self) -> (Node<T>, Vec<Box<NodeWithChildren<T>>>) {
//             let node = self.node;
//             let children = self
//                 .children
//                 .into_iter()
//                 .map(|child| Box::new(child.as_node_with_children()))
//                 .collect();
//
//             (node, children)
//         }
//     }
//
//     // TODO(fnichol): ok, that takes us back to the above impl, huh. So maybe we need to try the
//     // trait object version where the children are `Vec<Box<dyn Into<NodeWithChildren<_, ...>>>` or
//     // something like that?
// }
