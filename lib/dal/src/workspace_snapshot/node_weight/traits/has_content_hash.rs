use si_events::ContentHash;

use crate::workspace_snapshot::node_weight::{NodeWeightResult, *};

use super::StoresContent;

///
/// For node weights with a single content hash.
/// 
pub trait HasContentHash: StoresContent {
    fn content_hash(&self) -> ContentHash;
    fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()>;
}

macro_rules! impl_has_content_hash {
    ($type:ty) => {
        impl StoresContent for $type {
            fn content_store_hashes(&self) -> Vec<ContentHash> {
                self.content_store_hashes()
            }
        }
        impl HasContentHash for $type {
            fn content_hash(&self) -> ContentHash { self.content_hash() }
            fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
                self.new_content_hash(content_hash)
            }
        }
    };
}

impl_has_content_hash! { ComponentNodeWeight }
impl_has_content_hash! { ContentNodeWeight }
impl_has_content_hash! { FuncArgumentNodeWeight }
impl_has_content_hash! { FuncNodeWeight }
impl_has_content_hash! { PropNodeWeight }
impl_has_content_hash! { secret_node_weight::SecretNodeWeight }
