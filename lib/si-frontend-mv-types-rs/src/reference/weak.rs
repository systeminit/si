use std::marker::PhantomData;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::workspace_snapshot::{
    Checksum,
    ChecksumHasher,
};

use super::{
    ReferenceId,
    ReferenceKind,
};
use crate::checksum::FrontendChecksum;

/// A marker trait used for ensuring that weak references are strongly typed at compile time and
/// that the constructor method adheres to that type.
pub trait ReferenceKindMarker {
    const REFERENCE_KIND: ReferenceKind;
}

/// A weak reference is very similar to standard reference, but lacks a checksum.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, PartialOrd, Ord)]
pub struct WeakReference<T, R>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
    R: ReferenceKindMarker,
{
    pub kind: ReferenceKind,
    pub id: ReferenceId<T>,
    #[serde(skip)]
    _kind: PhantomData<R>,
}

impl<T, R> WeakReference<T, R>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
    R: ReferenceKindMarker,
{
    pub fn kind(&self) -> ReferenceKind {
        R::REFERENCE_KIND
    }
}

impl<T, R> WeakReference<T, R>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
    R: ReferenceKindMarker,
{
    pub fn new(id: T) -> Self {
        Self {
            kind: R::REFERENCE_KIND,
            id: ReferenceId(id),
            _kind: PhantomData,
        }
    }
}

impl<T, R> From<T> for WeakReference<T, R>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
    R: ReferenceKindMarker,
{
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T, R> FrontendChecksum for WeakReference<T, R>
where
    T: Eq + PartialEq + Clone + std::fmt::Debug + Serialize + std::fmt::Display,
    R: ReferenceKindMarker,
{
    fn checksum(&self) -> Checksum {
        // Note that the hasher does not include the checksum because a weak reference does not
        // include the checksum.
        let mut hasher = ChecksumHasher::new();
        hasher.update(FrontendChecksum::checksum(&self.kind).as_bytes());
        hasher.update(FrontendChecksum::checksum(&self.id.to_string()).as_bytes());
        hasher.finalize()
    }
}

/// Contains all markers needed for weak reference typing.
pub mod markers {
    use super::ReferenceKindMarker;
    use crate::reference::ReferenceKind;

    /// A weak reference marker for [`ReferenceKind::Component`].
    #[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct Component;

    impl ReferenceKindMarker for Component {
        const REFERENCE_KIND: ReferenceKind = ReferenceKind::Component;
    }

    /// A weak reference marker for [`ReferenceKind::IncomingConnections`].
    #[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct IncomingConnections;

    impl ReferenceKindMarker for IncomingConnections {
        const REFERENCE_KIND: ReferenceKind = ReferenceKind::IncomingConnections;
    }

    /// A weak reference marker for [`ReferenceKind::SchemaVariant`].
    #[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct SchemaVariant;

    impl ReferenceKindMarker for SchemaVariant {
        const REFERENCE_KIND: ReferenceKind = ReferenceKind::SchemaVariant;
    }

    /// A weak reference marker for [`ReferenceKind::SchemaMembers`].
    #[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
    pub struct SchemaMembers;

    impl ReferenceKindMarker for SchemaMembers {
        const REFERENCE_KIND: ReferenceKind = ReferenceKind::SchemaMembers;
    }
}
