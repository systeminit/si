use si_events::ContentHash;

use crate::{layer_db_types::ContentTypes, workspace_snapshot::{node_weight::{NodeWeightResult, *}, WorkspaceSnapshotResult}, DalContext, WorkspaceSnapshotError};

pub trait HasContentHash: AnyNodeWeight {
    fn content_hash(&self) -> ContentHash;
    fn content_store_hashes(&self) -> Vec<ContentHash>;
}
///
/// For node weights with a single content hash.
/// 
pub trait HasContent: HasContentHash {
    type ContentType: TryFrom<ContentTypes, Error: std::fmt::Display>;
    // fn read_content(&self, ctx: &DalContext) -> NodeWeightResult<Vec<u8>>;
    fn content_address(&self) -> ContentAddress;
    fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()>;
    fn content_address_discriminants(&self) -> ContentAddressDiscriminants {
        self.content_address().into()
    }
    async fn read_content(&self, ctx: &DalContext) -> WorkspaceSnapshotResult<Self::ContentType> {
        ctx
        .layer_db()
        .cas()
        .try_read_as(&self.content_hash())
        .await?
        .ok_or(WorkspaceSnapshotError::MissingContentFromStore(self.id()))
    }
}

macro_rules! impl_has_content {
    ($type:ty => $content_type:ty) => {
        impl $crate::workspace_snapshot::node_weight::HasContentHash for $type {
            fn content_store_hashes(&self) -> Vec<::si_events::ContentHash> {
                vec![self.content_address.content_hash()]
            }
            fn content_hash(&self) -> ::si_events::ContentHash { self.content_address.content_hash() }
        }
        impl $crate::workspace_snapshot::node_weight::HasContent for $type {
            type ContentType = $content_type;
            fn content_address(&self) -> $crate::workspace_snapshot::node_weight::ContentAddress { self.content_address }
            fn new_content_hash(&mut self, content_hash: ::si_events::ContentHash) -> $crate::workspace_snapshot::node_weight::NodeWeightResult<()> {
                self.content_address = <$content_type as $crate::layer_db_types::ContentType>::hash_into_address(content_hash);
                Ok(())
            }
        }
    };
}

pub(crate) use impl_has_content;
