use std::{sync::{Arc, Mutex}, fmt::Debug};

use error::CasResult;
use ulid::{Ulid, Generator};

mod error;

// * Function
//   * contentHash: ..
//   * vectorClock: [ ]
// * Schema
//   * contentHash ..
//   * vectorClock: [ ]
//   * references to every function it needs
// * Modules

#[derive(Clone)]
pub struct LamportClock {
    gen: Arc<Mutex<Generator>>,
    pub who: String,
    pub counter: Ulid,
}

impl LamportClock {
    pub fn new(who: impl Into<String>) -> LamportClock {
        let gen = Arc::new(Mutex::new(Generator::new()));
        let counter = gen.lock().unwrap().generate().unwrap();
        LamportClock {
            gen,
            who: who.into(),
            counter,
        }
    }

    pub fn inc(&mut self) {
        let next = self.gen.lock().unwrap().generate().unwrap();
        self.counter = next;
    }
}

impl Eq for LamportClock { }

impl PartialEq for LamportClock {
    fn eq(&self, other: &Self) -> bool {
        let who_is_eq = self.who == other.who;
        let counter_is_eq = self.counter == other.counter;
        who_is_eq && counter_is_eq
    }
}

impl PartialOrd for LamportClock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.who == other.who {
            self.counter.partial_cmp(&other.counter)
        } else {
            None
        }
    }
}

impl Debug for LamportClock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LamportClock")
            .field("who", &self.who)
            .field("counter", &self.counter)
            .finish()
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct VectorClock {
    pub id: Ulid,
    pub clock_entries: Vec<LamportClock>,
}

impl VectorClock {
    pub fn new(who: impl Into<String>) -> VectorClock {
        let lc = LamportClock::new(who);
        VectorClock {
            id: Ulid::new(),
            clock_entries: vec![lc],
        }
    }
}

#[derive(Debug, Default)]
pub struct Function {
    pub content_hash: String,
    pub vector_clock: VectorClock,
}

impl Function {
    pub fn new(content_hash: impl Into<String>) -> Function {
        Function {
            content_hash: content_hash.into(),
            vector_clock: VectorClock::new("poop"),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn lamport_clock_new() {
        let lc = LamportClock::new("adam");
        dbg!(lc);
    }

    #[test]
    fn lamport_clock_inc() {
        let lc = dbg!(LamportClock::new("adam"));
        let mut lc2 = lc.clone();
        assert_eq!(lc, lc2);
        //std::thread::sleep(std::time::Duration::from_secs(1));
        lc2.inc();
        dbg!(&lc);
        dbg!(&lc2);
        dbg!(lc.counter.to_string());
        dbg!(lc2.counter.to_string());
        assert_ne!(lc, lc2);
        assert!(lc < lc2);
    }

    #[test]
    fn lamport_clock_different_who() {
        let lc_adam = LamportClock::new("adam");
        let lc_nick = LamportClock::new("nick");
        assert_eq!(lc_adam.partial_cmp(&lc_nick), None);
        assert_eq!(lc_adam < lc_nick, false);
        assert_eq!(lc_adam > lc_nick, false);
    }

    #[test]
    fn vector_clock_new() {
        let vc = VectorClock::new("adam");
        assert_eq!(vc.clock_entries.len(), 1);
        assert_eq!(vc.clock_entries[0].who, "adam");
    }

}
