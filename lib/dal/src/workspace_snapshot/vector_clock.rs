//! Vector Clocks

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
    pub entries: HashMap<ChangeSetId, LamportClock>,
}

impl VectorClock {
    pub fn new(change_set: &ChangeSet) -> VectorClockResult<VectorClock> {
        let lamport_clock = LamportClock::new(change_set)?;
        let mut entries = HashMap::new();
        entries.insert(change_set.id, lamport_clock);

        Ok(VectorClock { entries })
    }

    pub fn inc(&mut self, change_set: &ChangeSet) -> VectorClockResult<()> {
        if let Some(lamport_clock) = self.entries.get_mut(&change_set.id) {
            lamport_clock.inc(change_set)?;
        } else {
            self.entries
                .insert(change_set.id, LamportClock::new(change_set)?);
        }

        Ok(())
    }

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

    pub fn fork(&self, change_set: &ChangeSet) -> VectorClockResult<VectorClock> {
        let mut forked = self.clone();
        forked.inc(change_set)?;

        Ok(forked)
    }
}

impl std::fmt::Debug for VectorClock {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.debug_map()
            .entries(self.entries.iter().map(|(k, v)| (k.to_string(), v)))
            .finish()
    }
}

//     // We are 'newer' than the other clock if we have seen all of the other clocks
//     // change sets, and we are newer than they are.
//     //
//     // TODO(nick,jacob): we need more in place to solve this.
//     pub fn is_newer(&self, other: &VectorClock) -> bool {
//         let mut is_newer = true;
//         for (other_change_set_id, other_lamport_clock) in other.entries {
//             if let Some(local_lamport_clock) = self.entries.get(&other_change_set_id) {
//                 if local_lamport_clock < &other_lamport_clock {
//                     is_newer = false;
//                 }
//             } else {
//                 is_newer = false;
//             }
//         }
//         is_newer
//     }
//
//     pub fn newer_for_change_set(&self, change_set_pk: ChangeSetPk, other: &VectorClock) -> bool {
//         let is_newer = false;
//         if let Some(my_lc) = self.entries.get(&change_set_pk) {
//             if let Some(other_lc) = other.entries.get(&change_set_pk) {
//                 return my_lc > other_lc;
//             }
//         }
//         is_newer
//     }
//
//     // The clock was changed if there is an entry in the vector for a change set pk
//     pub fn was_changed_in_changeset(&self, change_set_pk: ChangeSetPk) -> bool {
//         self.entries.get(&change_set_pk).is_some()
//     }
// }
//
// #[cfg(test)]
// mod test {
//     use super::*;
//
//     #[test]
//     fn already_seen() {
//         let object_id = Ulid::new();
//         let mut vector_clock_a = VectorClock::new(object_id, ChangeSetPk::new());
//         let vector_clock_b = vector_clock_a.fork(ChangeSetPk::new()).unwrap();
//         assert!(vector_clock_b.is_newer(&vector_clock_a));
//         assert!(!vector_clock_a.is_newer(&vector_clock_b));
//         let change_set_pk = ChangeSetPk::new();
//         vector_clock_a
//             .merge(change_set_pk, &vector_clock_b)
//             .unwrap();
//         assert!(vector_clock_a.is_newer(&vector_clock_b));
//     }
// }
