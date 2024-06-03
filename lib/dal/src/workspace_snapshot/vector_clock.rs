//! Vector Clocks

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::pk;
use crate::workspace_snapshot::lamport_clock::{LamportClock, LamportClockError};

#[derive(Debug, Error)]
pub enum VectorClockError {
    #[error("Lamport Clock Error: {0}")]
    LamportClock(#[from] LamportClockError),
}

const CLOCKS_TO_RETAIN: usize = 7;

pub type VectorClockResult<T> = Result<T, VectorClockError>;

pk!(VectorClockId);

#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct VectorClock {
    entries: HashMap<VectorClockId, LamportClock>,
}

impl VectorClock {
    /// Create a new [`VectorClock`] with an entry for [`VectorClockId`].
    pub fn new(vector_clock_id: VectorClockId) -> VectorClockResult<VectorClock> {
        let lamport_clock = LamportClock::new()?;
        let mut entries = HashMap::new();
        entries.insert(vector_clock_id, lamport_clock);

        Ok(VectorClock { entries })
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
    pub fn inc(&mut self, vector_clock_id: VectorClockId) -> VectorClockResult<()> {
        if let Some(lamport_clock) = self.entries.get_mut(&vector_clock_id) {
            lamport_clock.inc()?;
        } else {
            self.entries.insert(vector_clock_id, LamportClock::new()?);
        }

        Ok(())
    }

    /// Add all entries in `other` to `self`, taking the most recent value if the entry already
    /// exists in `self`.
    pub fn merge(&mut self, other: &VectorClock) -> VectorClockResult<()> {
        for (other_vector_clock_id, other_lamport_clock) in other.entries.iter() {
            if let Some(lamport_clock) = self.entries.get_mut(other_vector_clock_id) {
                lamport_clock.merge(other_lamport_clock);
            } else {
                self.entries
                    .insert(*other_vector_clock_id, *other_lamport_clock);
            }
        }

        Ok(())
    }

    /// Return a new [`VectorClock`] with the entry for [`VectorClockId`] incremented.
    pub fn fork(&self, vector_clock_id: VectorClockId) -> VectorClockResult<VectorClock> {
        let mut forked = self.clone();
        forked.inc(vector_clock_id)?;

        Ok(forked)
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

    pub fn remove_entries(&mut self, allow_list: &[VectorClockId]) {
        let mut removed_clocks = vec![];
        self.entries.retain(|clock_id, &mut clock| {
            let retain = allow_list.contains(clock_id);
            if !retain {
                removed_clocks.push((*clock_id, clock));
            }
            retain
        });

        removed_clocks.sort_by(|&(_, clock_a), &(_, clock_b)| clock_a.cmp(&clock_b));
        // retain some max clocks
        for (clock_id, clock) in removed_clocks.into_iter().rev().take(CLOCKS_TO_RETAIN) {
            self.inc_to(clock_id, clock.counter);
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
