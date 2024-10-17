//! ## SiVersionedNodeWeight
//!
//! Derive macro for the versioned wrapper types used for node weights in
//! [`WorkspaceSnapshotGraph`][::dal::workspace_snapshot::graph::WorkspaceSnapshotGraph]. The enum
//! variant for the "current" version of the node weight must be annotated
//! `#[si_versioned_node_weight(current)]`, and be a newtype style variant. The macro checks to
//! make sure that exactly one variant is marked as "current", and that it has the correct shape.
//!
//! ```ignore
//! #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SiVersionedNodeWeight)]
//! pub enum SomeNodeWeight {
//!     V0,
//!     V1(SomeNodeWeightV1),
//!     #[si_versioned_node_weight(current)]
//!     V2(SomeNodeWeightV2)
//! }
//! ```
//!
//! This would generate the following `impl`:
//!
//! ```ignore
//! impl SiVersionedNodeWeight for SomeNodeWeight {
//!     type Inner = SomeNodeWeightV2;
//!
//!     fn inner(&self) -> &Self::Inner {
//!         match self {
//!             Self::V2(inner) => inner,
//!             _ => {
//!                 panic!("Attempted to get reference to unsupported SomeNodeWeight variant");
//!             }
//!         }
//!     }
//!
//!     fn inner_mut(&mut self) -> &mut Self::Inner {
//!         match self {
//!             Self::V2(inner) => inner,
//!             _ => {
//!                 panic!("Attempted to get mutable reference to unsupported SomeNodeWeight variant");
//!             }
//!         }
//!     }
//! }
//! ```
//!
//! ## SiNodeWeight
//!
//! Derive macro for the individual versions of a node weight used in `WorkspaceSnapshotGraph`. The
//! associated `NodeWeightDiscriminants` to expose through the `node_weight_discriminant` method
//! must be specified.
//!
//! ```ignore
//! #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SiNodeWeight)]
//! #[si_node_weight(discriminant = NodeWeightDiscriminants::SomeNodeWeight)]
//! pub struct SomeNodeWeightV2 {
//!     id: Ulid,
//!     lineage_id: LineageId,
//!     merkle_tree_hash: MerkleTreeHash,
//!     #[si_node_weight(node_hash)]
//!     some_string: String,
//!     #[si_node_weight(node_hash = "self.content_address.content_hash().as_bytes()")]
//!     content_address: ContentAddress,
//!     timestamp: Timestamp,
//! }
//! ```
//!
//! ### Derived Methods
//!
//! Any automically derived method can be overridden, and manually implemented by listing that
//! method in the list of methods to skip.
//!
//! ```ignore
//! #[derive(SiNodeWeight)]
//! #[si_node_weight(skip("id", "lineage_id", "set_merkle_tree_hash"))]
//! pub struct SomeNodeWeightV2 { ... }
//!
//! impl SiNodeWeightV2 {
//!     pub fn id(&self) -> Ulid { ... }
//!     pub fn lineage_id(&self) -> LineageId { ... }
//!     pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) { ... }
//! }
//! ```
//!
//! #### content_hash
//!
//! If the struct has a `content_address` field, the derived method will return
//! `self.content_address.content_hash()`, otherwise it will return `self.node_hash()`.
//!
//! #### id
//!
//! Derived method returns `self.id`.
//!
//! #### lineage_id
//!
//! Derived method returns `self.lineage_id`.
//!
//! #### merkle_tree_hash
//!
//! Derived method returns `self.merkle_tree_hash`.
//!
//! #### node_hash
//!
//! Derived method will use any fields with the `#[si_node_weight(node_hash)]` annotation as part
//! of the node hash calculation. By default it will try to call `.as_bytes()` on the field
//! (`self.field_name.as_bytes()`). If the field does not hav an `.as_bytes()` method, the field
//! should be annotated with how to get a `&[u8]` from it (`#[si_node_weight(node_hash `
//! "self.arity.to_string().as_bytes()")]`). The fields will be fed to a `ContentHasher` in the
//! order they appear in the struct.
//!
//! ```ignore
//! #[derive(SiNodeWeight)]
//! pub struct SomeNodeWeightV2 {
//!     #[si_node_weight(node_hash)]
//!     field_a: SomeTypeWithAsBytes,
//!     #[si_node_weight(node_hash = "&[self.field_b]")]
//!     field_b: u8,
//!     field_c: u16,
//!     #[si_node_weight(node_hash = "self.field_d.to_string().as_bytes()")]
//!     field_d: SomeTypeWithoutAsBytes,
//! }
//! ```
//!
//! This would generate the following `node_hash` method:
//!
//! ```ignore
//! pub fn node_hash(&self) -> ContentHash {
//!     let mut content_hasher = ContentHash::hasher();
//!     content_hasher.update(self.field_a);
//!     content_hasher.update(&[self.field_b]);
//!     content_hasher.update(self.field_d.to_string().as_bytes());
//!
//!     content_hasher.finalize()
//! }
//! ```
//!
//! #### node_weight_discriminants
//!
//! Generated from the `NodeWeightDiscriminants` specified in the struct annotation.
//!
//! ```ignore
//! #[derive(SiNodeWeight)]
//! #[si_node_weight(discriminant = NodeWeightDiscriminants::SomeNodeWeight)]
//! pub struct SomeNodeWeightV2 { ... }
//! ```
//!
//! #### set_id
//!
//! Derived method assigns to `self.id`.
//!
//! #### set_lineage_id
//!
//! Derived method assigns to `self.lineage_id`.
//!
//! #### set_merkle_tree_hash
//!
//! Derived method assigns to `self.merkle_tree_hash`.

use manyhow::manyhow;

mod node_weight;

#[manyhow]
#[proc_macro_derive(SiVersionedNodeWeight, attributes(si_versioned_node_weight))]
pub fn si_versioned_node_weight(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    node_weight::versioned::derive_si_versioned_node_weight(input, errors)
}

#[manyhow]
#[proc_macro_derive(SiNodeWeight, attributes(si_node_weight))]
pub fn si_node_weight(
    input: proc_macro::TokenStream,
    errors: &mut manyhow::Emitter,
) -> manyhow::Result<proc_macro::TokenStream> {
    node_weight::derive_si_node_weight(input, errors)
}
