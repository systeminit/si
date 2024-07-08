//! Vector Clocks

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::workspace_snapshot::lamport_clock::{LamportClock, LamportClockError};
use crate::{pk, ChangeSetId};

pub use si_events::{VectorClockActorId, VectorClockChangeSetId, VectorClockId};

#[derive(Debug, Error)]
pub enum VectorClockError {
    #[error("Lamport Clock Error: {0}")]
    LamportClock(#[from] LamportClockError),
}

pub type VectorClockResult<T> = Result<T, VectorClockError>;

pk!(DeprecatedVectorClockId);

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct VectorClock {
    entries: HashMap<VectorClockId, LamportClock>,
}

impl VectorClock {
    /// Create a new [`VectorClock`] with an entry for [`VectorClockId`].
    pub fn new(vector_clock_id: VectorClockId) -> Self {
        let lamport_clock = LamportClock::new();
        let mut entries = HashMap::new();
        entries.insert(vector_clock_id, lamport_clock);

        VectorClock { entries }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn empty() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn max(
        &self,
        change_set_id_filter: Option<ChangeSetId>,
    ) -> Option<(VectorClockId, LamportClock)> {
        let maybe_change_set_id = change_set_id_filter
            .map(|change_set_id| VectorClockChangeSetId::new(change_set_id.into_inner().into()));
        self.entries
            .iter()
            .filter(|(clock_id, _)| {
                maybe_change_set_id
                    .map(|vc_cs_id| clock_id.change_set_id() == vc_cs_id)
                    .unwrap_or(true)
            })
            .max_by(|(_, clock_a), (_, clock_b)| (**clock_a).cmp(*clock_b))
            .map(|(clock_id, clock)| (*clock_id, *clock))
    }

    pub fn entry_for(&self, vector_clock_id: VectorClockId) -> Option<LamportClock> {
        self.entries.get(&vector_clock_id).copied()
    }

    pub fn has_entries_newer_than(&self, clock_stamp: LamportClock) -> bool {
        self.entries.values().any(|v| *v > clock_stamp)
    }

    pub fn inc_to(&mut self, vector_clock_id: VectorClockId, new_clock_value: DateTime<Utc>) {
        if let Some(lamport_clock) = self.entries.get_mut(&vector_clock_id) {
            lamport_clock.inc_to(new_clock_value);
        } else {
            self.entries.insert(
                vector_clock_id,
                LamportClock::new_with_value(new_clock_value),
            );
        }
    }

    /// Increment the entry for [`VectorClockId`], adding one if there wasn't one already.
    pub fn inc(&mut self, vector_clock_id: VectorClockId) {
        if let Some(lamport_clock) = self.entries.get_mut(&vector_clock_id) {
            lamport_clock.inc();
        } else {
            self.entries.insert(vector_clock_id, LamportClock::new());
        }
    }

    /// Add all entries in `other` to `self`, taking the most recent value if the entry already
    /// exists in `self`, then increment the entry for [`VectorClockId`] (adding one if it is not
    /// already there).
    pub fn merge(&mut self, vector_clock_id: VectorClockId, other: &VectorClock) {
        for (other_vector_clock_id, other_lamport_clock) in other.entries.iter() {
            if let Some(lamport_clock) = self.entries.get_mut(other_vector_clock_id) {
                lamport_clock.merge(other_lamport_clock);
            } else {
                self.entries
                    .insert(*other_vector_clock_id, *other_lamport_clock);
            }
        }
        self.inc(vector_clock_id);
    }

    /// Return a new [`VectorClock`] with the entry for [`VectorClockId`] incremented.
    pub fn fork(&self, vector_clock_id: VectorClockId) -> VectorClock {
        let mut forked = self.clone();
        forked.inc(vector_clock_id);

        forked
    }

    /// Returns true if all entries in `other` are present in `self`, and `<=` the entry in
    /// `self`, meaning that `self` has already seen/incorporated all of the information
    /// in `other`.
    pub fn is_newer_than(&self, other: &VectorClock) -> bool {
        for (other_vector_clock_id, other_lamport_clock) in &other.entries {
            if let Some(my_clock) = self.entries.get(other_vector_clock_id) {
                if other_lamport_clock > my_clock {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    pub fn get_shared_clock_ids(&self, other: &HashSet<VectorClockId>) -> HashSet<VectorClockId> {
        let entry_set = HashSet::from_iter(self.entries.keys().map(ToOwned::to_owned));

        entry_set
            .intersection(other)
            .map(ToOwned::to_owned)
            .collect()
    }

    /// Remove all vector clock entries except those in `allow_list` and
    /// collapse them into the collapse_id by choosing the maximum removed entry
    pub fn collapse_entries(
        &mut self,
        allow_list: &HashSet<VectorClockChangeSetId>,
        collapse_id: VectorClockId,
    ) {
        let mut max_removed = None;
        self.entries.retain(|clock_id, &mut lamport_clock| {
            if allow_list.contains(&clock_id.change_set_id()) {
                true
            } else {
                if Some(lamport_clock) > max_removed {
                    max_removed = Some(lamport_clock.to_owned());
                }

                false
            }
        });

        if let Some(max_removed) = max_removed {
            self.inc_to(collapse_id, max_removed.counter);
        }
    }
}

impl std::fmt::Debug for VectorClock {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_map()
            .entries(self.entries.iter().map(|(k, v)| (k.to_string(), v)))
            .finish()
    }
}

pub trait HasVectorClocks {
    fn vector_clock_first_seen(&self) -> &VectorClock;
    fn vector_clock_first_seen_mut(&mut self) -> &mut VectorClock;

    fn vector_clock_recently_seen(&self) -> &VectorClock;
    fn vector_clock_recently_seen_mut(&mut self) -> &mut VectorClock;

    fn vector_clock_write(&self) -> &VectorClock;
    fn vector_clock_write_mut(&mut self) -> &mut VectorClock;

    fn increment_vector_clocks(&mut self, vector_clock_id: VectorClockId) {
        self.vector_clock_write_mut().inc(vector_clock_id);
        self.vector_clock_recently_seen_mut().inc(vector_clock_id);
    }

    fn new_with_incremented_vector_clock(&self, vector_clock_id: VectorClockId) -> Self
    where
        Self: Sized + Clone,
    {
        let mut new_self = self.clone();
        new_self.increment_vector_clocks(vector_clock_id);

        new_self
    }

    fn mark_seen_at(&mut self, vector_clock_id: VectorClockId, seen_at: DateTime<Utc>) {
        self.vector_clock_recently_seen_mut()
            .inc_to(vector_clock_id, seen_at);
        if self
            .vector_clock_first_seen()
            .entry_for(vector_clock_id)
            .is_none()
        {
            self.vector_clock_first_seen_mut()
                .inc_to(vector_clock_id, seen_at);
        }
    }
}
