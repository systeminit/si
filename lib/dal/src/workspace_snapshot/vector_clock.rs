//! Vector Clocks

use std::collections::HashMap;

use crate::workspace_snapshot::lamport_clock::LamportClock;
use crate::workspace_snapshot::{ChangeSet, ChangeSetId};

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct VectorClock {
    pub entries: HashMap<ChangeSetId, LamportClock>,
}

impl VectorClock {
    pub fn new(change_set: &ChangeSet) -> VectorClock {
        let lamport_clock = LamportClock::new(change_set);
        let mut entries = HashMap::new();
        entries.insert(change_set.id, lamport_clock);
        VectorClock { entries }
    }

    pub fn inc(&mut self, change_set: &ChangeSet) {
        self.entries
            .entry(change_set.id)
            .and_modify(|lc| lc.inc(change_set))
            .or_insert(LamportClock::new(change_set));
    }

    pub fn merge(&mut self, change_set: &ChangeSet, other: &VectorClock) {
        for (other_change_set_id, other_lamport_clock) in other.entries.iter() {
            self.entries
                .entry(*other_change_set_id)
                .and_modify(|local_lamport_clock| local_lamport_clock.merge(other_lamport_clock))
                .or_insert(other_lamport_clock.clone());
        }
        self.inc(change_set);
    }

    pub fn fork(&self, change_set: &ChangeSet) -> VectorClock {
        let mut forked = self.clone();
        forked.inc(change_set);
        forked
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
