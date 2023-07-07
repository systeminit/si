use std::collections::HashMap;

use ulid::Ulid;

use crate::{
    change_set::ChangeSetPk,
    error::{DagError, DagResult},
    lamport_clock::LamportClock,
};

// We keep a vector clock of every changeset that has impacted our given object id
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct VectorClock {
    pub object_id: Ulid,
    pub clock_entries: HashMap<ChangeSetPk, LamportClock>,
}

impl VectorClock {
    pub fn new(object_id: Ulid, change_set_pk: ChangeSetPk) -> VectorClock {
        let lc = LamportClock::new(change_set_pk);
        let mut clock_entries = HashMap::new();
        clock_entries.insert(change_set_pk, lc);
        VectorClock {
            object_id,
            clock_entries,
        }
    }

    pub fn inc(&mut self, change_set_pk: ChangeSetPk) {
        self.clock_entries
            .entry(change_set_pk)
            .and_modify(|lc| lc.inc())
            .or_insert(LamportClock::new(change_set_pk));
    }

    pub fn merge(&mut self, change_set_pk: ChangeSetPk, other: &VectorClock) -> DagResult<()> {
        if self.object_id != other.object_id {
            dbg!(&self);
            dbg!(&other);
            return Err(DagError::CannotMergeVectorClocksForDifferentObjects);
        }
        for (other_key, other_value) in other.clock_entries.iter() {
            self.clock_entries
                .entry(*other_key)
                .and_modify(|my_value| my_value.merge(other_value))
                .or_insert(other_value.clone());
        }
        self.inc(change_set_pk);
        Ok(())
    }

    pub fn fork(&self, change_set_pk: ChangeSetPk) -> DagResult<VectorClock> {
        let mut forked = self.clone();
        forked.inc(change_set_pk);
        Ok(forked)
    }

    // We are 'newer' than the other clock if we have seen all of the other clocks
    // change sets, and we are newer than they are.
    pub fn is_newer(&self, other: &VectorClock) -> bool {
        let mut is_newer = true;
        for other_clock in other.clock_entries.values() {
            if let Some(my_clock) = self.clock_entries.get(&other_clock.change_set_pk) {
                if my_clock < other_clock {
                    is_newer = false;
                }
            } else {
                is_newer = false;
            }
        }
        is_newer
    }

    pub fn newer_for_change_set(&self, change_set_pk: ChangeSetPk, other: &VectorClock) -> bool {
        let is_newer = false;
        if let Some(my_lc) = self.clock_entries.get(&change_set_pk) {
            if let Some(other_lc) = other.clock_entries.get(&change_set_pk) {
                return my_lc > other_lc;
            }
        }
        is_newer
    }

    // The clock was changed if there is an entry in the vector for a change set pk
    pub fn was_changed_in_changeset(&self, change_set_pk: ChangeSetPk) -> bool {
        self.clock_entries.get(&change_set_pk).is_some()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn already_seen() {
        let object_id = Ulid::new();
        let mut vector_clock_a = VectorClock::new(object_id, ChangeSetPk::new());
        let vector_clock_b = vector_clock_a.fork(ChangeSetPk::new()).unwrap();
        assert!(vector_clock_b.is_newer(&vector_clock_a));
        assert!(!vector_clock_a.is_newer(&vector_clock_b));
        let change_set_pk = ChangeSetPk::new();
        vector_clock_a
            .merge(change_set_pk, &vector_clock_b)
            .unwrap();
        assert!(vector_clock_a.is_newer(&vector_clock_b));
    }
}
