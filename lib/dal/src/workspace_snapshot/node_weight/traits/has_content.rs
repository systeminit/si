use si_events::ContentHash;

use crate::workspace_snapshot::node_weight::{NodeWeightResult, *};


pub trait HasContent {
    fn content_hash(&self) -> ContentHash;
    fn content_store_hashes(&self) -> Vec<ContentHash>;
}

///
/// For node weights with a single content hash.
/// 
pub trait HasContentAddress: HasContent {
    fn content_address(&self) -> ContentAddress;
    fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()>;
    fn content_address_discriminants(&self) -> ContentAddressDiscriminants {
        self.content_address().into()
    }
}

pub trait HasDiscriminatedContentAddress: HasContentAddress {
    const CONTENT_ADDRESS_DISCRIMINANT: ContentAddressDiscriminants;
}

macro_rules! impl_has_discriminated_content_address {
    ($type:ty: $discriminant:ident) => {
        impl $crate::workspace_snapshot::node_weight::HasContent for $type {
            fn content_store_hashes(&self) -> Vec<::si_events::ContentHash> {
                vec![self.content_address.content_hash()]
            }
            fn content_hash(&self) -> ::si_events::ContentHash { self.content_address.content_hash() }
        }
        impl $crate::workspace_snapshot::node_weight::HasContentAddress for $type {
            fn content_address(&self) -> $crate::workspace_snapshot::node_weight::ContentAddress { self.content_address }
            fn new_content_hash(&mut self, content_hash: ::si_events::ContentHash) -> $crate::workspace_snapshot::node_weight::NodeWeightResult<()> {
                let new_address = match &self.content_address {
                    $crate::workspace_snapshot::node_weight::ContentAddress::$discriminant(_) => $crate::workspace_snapshot::node_weight::ContentAddress::$discriminant(content_hash),
                    other => {
                        return Err($crate::workspace_snapshot::node_weight::NodeWeightError::InvalidContentAddressForWeightKind(
                            Into::<$crate::workspace_snapshot::ContentAddressDiscriminants>::into(other).to_string(),
                            $crate::workspace_snapshot::ContentAddressDiscriminants::$discriminant.to_string(),
                        ));
                    }
                };
        
                self.content_address = new_address;
        
                Ok(())
            }
        }
        impl $crate::workspace_snapshot::node_weight::HasDiscriminatedContentAddress for $type {
            const CONTENT_ADDRESS_DISCRIMINANT: $crate::workspace_snapshot::ContentAddressDiscriminants = $crate::workspace_snapshot::ContentAddressDiscriminants::$discriminant;
        }
    };
}

pub(crate) use impl_has_discriminated_content_address;
