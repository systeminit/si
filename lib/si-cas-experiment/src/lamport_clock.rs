use std::sync::{Arc, Mutex};

use ulid::{Generator, Ulid};

use crate::change_set::ChangeSetPk;

#[derive(Clone)]
pub struct LamportClock {
    gen: Arc<Mutex<Generator>>,
    pub change_set_pk: ChangeSetPk,
    pub counter: Ulid,
}

impl LamportClock {
    pub fn new(change_set_pk: ChangeSetPk) -> LamportClock {
        let gen = Arc::new(Mutex::new(Generator::new()));
        let counter = gen.lock().unwrap().generate().unwrap();
        LamportClock {
            gen,
            change_set_pk,
            counter,
        }
    }

    pub fn inc(&mut self) {
        let next = self.gen.lock().unwrap().generate().unwrap();
        self.counter = next;
    }

    pub fn merge(&mut self, other: &LamportClock) {
        if self.change_set_pk == other.change_set_pk && self.counter < other.counter {
            self.counter = other.counter;
        }
    }
}

impl Eq for LamportClock {}

impl PartialEq for LamportClock {
    fn eq(&self, other: &Self) -> bool {
        let change_set_is_eq = self.change_set_pk == other.change_set_pk;
        let counter_is_eq = self.counter == other.counter;
        change_set_is_eq && counter_is_eq
    }
}

impl PartialOrd for LamportClock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.change_set_pk == other.change_set_pk {
            self.counter.partial_cmp(&other.counter)
        } else {
            None
        }
    }
}

impl std::fmt::Debug for LamportClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LamportClock")
            .field("change_set_pk", &self.change_set_pk)
            .field("counter", &self.counter)
            .finish()
    }
}

