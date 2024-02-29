use moka::future::Cache as MokaAsyncCache;
use std::time::Duration;

use crate::change_set_pointer::ChangeSetPointerId;

use super::{WorkspaceSnapshot, WorkspaceSnapshotId};

type WorkspaceSnapshotCache = MokaAsyncCache<WorkspaceSnapshotId, WorkspaceSnapshot>;
type ChangeSetPointerCache = MokaAsyncCache<ChangeSetPointerId, WorkspaceSnapshotId>;

/// The max capacity of the cache. LRU/LFU logic will be used to evict items
/// when this max is reached.
static DEFAULT_MAX_CAPACITY: u64 = 32_768;

/// The Time To Live determines how long a cache entry is valid for. No matter
/// how many times it is accessed, the cache entry will be evicted after TTL
/// seconds. This is a 100 day default, which is probably too long given how
/// frequently the snapshot will be updated.
static DEFAULT_TTL: Duration = Duration::from_secs(60 * 60 * 24 * 100);

/// TTI is the Time To Idle. If a cache entry is not accessed for TTI seconds,
/// it will be evicted.  If it is accessed again, the idle clock begins again,
/// but no matter what it will be evicted after the TTL.  This is a 7 day
/// default, which is also probably too long.
static DEFAULT_TTI: Duration = Duration::from_secs(60 * 60 * 24 * 7);

/// Shared cache of snapshots, change set pointers, node weights, etc.  Moka is
/// used as the concurrent, lock-free hashmap for caching different types. The
/// moka cache is cloneable like an Arc, so it can be shared between
/// threads/tasks (it is Send and Sync). Types added to this struct should be
/// cloneable in the same way, and ideally should also be lock free
#[derive(Clone, Debug)]
pub struct Cache {
    snapshots: WorkspaceSnapshotCache,
}

impl Cache {
    pub fn snapshots(&self) -> &WorkspaceSnapshotCache {
        &self.snapshots
    }
}

impl Default for Cache {
    fn default() -> Self {
        let snapshots = WorkspaceSnapshotCache::builder()
            .max_capacity(DEFAULT_MAX_CAPACITY)
            .time_to_live(DEFAULT_TTL)
            .time_to_idle(DEFAULT_TTI)
            .build();

        Self { snapshots }
    }
}
