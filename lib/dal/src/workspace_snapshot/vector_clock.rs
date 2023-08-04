//! Vector Clocks

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use ulid::Ulid;

use crate::workspace_snapshot::{
    lamport_clock::{LamportClock, LamportClockError},
    {ChangeSet, ChangeSetId},
};

#[derive(Debug, Error)]
pub enum VectorClockError {
    #[error("Lamport Clock Error: {0}")]
    LamportClock(#[from] LamportClockError),
}

pub type VectorClockResult<T> = Result<T, VectorClockError>;

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct VectorClock {
    entries: HashMap<ChangeSetId, LamportClock>,
}

impl VectorClock {
    /// Create a new [`VectorClock`] with an entry for [`ChangeSet`].
    pub fn new(change_set: &ChangeSet) -> VectorClockResult<VectorClock> {
        let lamport_clock = LamportClock::new()?;
        let mut entries = HashMap::new();
        entries.insert(change_set.id, lamport_clock);

        Ok(VectorClock { entries })
    }

    pub fn entry_for(&self, change_set: &ChangeSet) -> Option<LamportClock> {
        self.entries.get(&change_set.id).copied()
    }

    pub fn has_entries_newer_than(&self, clock_stamp: LamportClock) -> bool {
        self.entries.values().any(|v| *v > clock_stamp)
    }

    pub fn inc_to(&mut self, change_set: &ChangeSet, new_clock_value: DateTime<Utc>) {
        if let Some(lamport_clock) = self.entries.get_mut(&change_set.id) {
            lamport_clock.inc_to(new_clock_value);
        } else {
            self.entries
                .insert(change_set.id, LamportClock::new_with_value(new_clock_value));
        }
    }

    /// Increment the entry for [`ChangeSet`], adding one if there wasn't one already.
    pub fn inc(&mut self, change_set: &ChangeSet) -> VectorClockResult<()> {
        if let Some(lamport_clock) = self.entries.get_mut(&change_set.id) {
            lamport_clock.inc()?;
        } else {
            self.entries.insert(change_set.id, LamportClock::new()?);
        }

        Ok(())
    }

    /// Add all entries in `other` to `self`, taking the most recent value if the entry already
    /// exists in `self`, then increment the entry for [`ChangeSet`] (adding one if it is not
    /// already there).
    pub fn merge(&mut self, change_set: &ChangeSet, other: &VectorClock) -> VectorClockResult<()> {
        for (other_change_set_id, other_lamport_clock) in other.entries.iter() {
            if let Some(lamport_clock) = self.entries.get_mut(other_change_set_id) {
                lamport_clock.merge(other_lamport_clock);
            } else {
                self.entries
                    .insert(*other_change_set_id, *other_lamport_clock);
            }
        }
        self.inc(change_set)?;

        Ok(())
    }

    /// Return a new [`VectorClock`] with the entry for [`ChangeSet`] incremented.
    pub fn fork(&self, change_set: &ChangeSet) -> VectorClockResult<VectorClock> {
        let mut forked = self.clone();
        forked.inc(change_set)?;

        Ok(forked)
    }

    /// Returns true if all entries in `other` are present in `self`, and `<=` the entry in
    /// `self`, meaning that `self` has already seen/incorporated all of the information
    /// in `other`.
    pub fn is_newer_than(&self, other: &VectorClock) -> bool {
        for (other_change_set_id, other_lamport_clock) in &other.entries {
            if let Some(my_clock) = self.entries.get(other_change_set_id) {
                if other_lamport_clock > my_clock {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }
}

impl std::fmt::Debug for VectorClock {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_map()
            .entries(self.entries.iter().map(|(k, v)| (k.to_string(), v)))
            .finish()
    }
}
