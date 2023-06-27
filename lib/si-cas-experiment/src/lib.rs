use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use error::{CasError, CasResult};
use ulid::{Generator, Ulid};

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

    pub fn merge(&mut self, other: &LamportClock) {
        if self.who == other.who && self.counter < other.counter {
            self.counter = other.counter.clone();
        }
    }
}

impl Eq for LamportClock {}

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
    pub who: Option<String>,
    pub clock_entries: HashMap<String, LamportClock>,
}

// F
// F A: A0
// F B: A0 B0
// F B -> A
// F A,B: A0 B0 A1

// Each function in each workspace gets a vector clock that shows its history
impl VectorClock {
    pub fn new(who: impl Into<String>) -> VectorClock {
        let who = who.into();
        let lc = LamportClock::new(who.clone());
        let mut clock_entries = HashMap::new();
        clock_entries.insert(who.clone(), lc);
        VectorClock {
            id: Ulid::new(),
            who: Some(who),
            clock_entries,
        }
    }

    pub fn inc(&mut self) -> CasResult<()> {
        self.clock_entries
            .entry(self.who.clone().ok_or(CasError::NoWho)?)
            .and_modify(|lc| lc.inc())
            .or_insert(LamportClock::new(self.who.as_ref().unwrap()));
        Ok(())
    }

    pub fn merge(&mut self, other: &VectorClock) -> CasResult<()> {
        if self.id != other.id {
            return Err(CasError::WrongMergeId);
        }
        for (other_key, other_value) in other.clock_entries.iter() {
            self.clock_entries
                .entry(other_key.to_string())
                .and_modify(|my_value| my_value.merge(other_value))
                .or_insert(other_value.clone());
        }
        self.inc()?;
        Ok(())
    }

    pub fn fork(&self, who: impl Into<String>) -> CasResult<VectorClock> {
        let mut forked = self.clone();
        let who = who.into();
        forked.who = Some(who.clone());
        forked.inc()?;
        Ok(forked)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Function {
    pub content_hash: String,
    pub vector_clock: VectorClock,
}

impl Function {
    pub fn new(content_hash: impl Into<String>, who: impl Into<String>) -> Function {
        Function {
            content_hash: content_hash.into(),
            vector_clock: VectorClock::new(who),
        }
    }

    pub fn id(&self) -> &Ulid {
        &self.vector_clock.id
    }

    pub fn update(
        &mut self,
        content_hash: impl Into<String>,
        who: impl Into<String>,
    ) -> CasResult<()> {
        self.content_hash = content_hash.into();
        self.vector_clock = self.vector_clock.fork(who)?;
        Ok(())
    }

    pub fn merge(&mut self, func: &Function) -> CasResult<()> {
        self.vector_clock.merge(&func.vector_clock)?;
        self.content_hash = func.content_hash.clone();
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct Module {
    pub id: Ulid,
    pub name: String,
    pub funcs: HashMap<Ulid, Function>,
}

impl Module {
    pub fn new(name: impl Into<String>) -> Module {
        Module {
            id: Ulid::new(),
            name: name.into(),
            funcs: HashMap::new(),
        }
    }

    pub fn add(&mut self, func: Function) {
        self.funcs.insert(func.id().clone(), func);
    }

    pub fn function(&mut self, function_id: &Ulid) -> CasResult<&mut Function> {
        let func = self
            .funcs
            .get_mut(function_id)
            .ok_or(CasError::NoContentHash)?;
        Ok(func)
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
        lc2.inc();
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
        assert!(vc.clock_entries.get("adam").is_some());
    }

    #[test]
    fn vector_clock_inc() {
        let mut vc = VectorClock::new("adam");
        let lc_og = vc.clock_entries.get("adam").unwrap().clone();
        vc.inc().unwrap();
        assert!(&lc_og < vc.clock_entries.get("adam").unwrap());
    }

    #[test]
    fn vector_clock_merge() {
        let mut vc_adam = VectorClock::new("adam");
        let vc_adam_og = vc_adam.clock_entries.get("adam").unwrap().clone();
        let mut vc_jacob = vc_adam.fork("jacob").unwrap();
        let vc_jacob_og = vc_jacob.clock_entries.get("jacob").unwrap().clone();

        assert!(vc_jacob.clock_entries.get("jacob").is_some());
        assert!(vc_jacob.clock_entries.get("adam").is_some());

        vc_jacob.merge(&vc_adam).unwrap();

        assert_eq!(vc_jacob.clock_entries.get("adam").unwrap(), &vc_adam_og);
        assert!(vc_jacob.clock_entries.get("jacob").unwrap() > &vc_jacob_og);

        vc_adam.inc().unwrap();

        vc_jacob.merge(&vc_adam).unwrap();
        assert_eq!(
            &vc_jacob.clock_entries.get("adam").unwrap(),
            &vc_adam.clock_entries.get("adam").unwrap()
        );
    }

    #[test]
    fn vector_clock_complex_merge() {
        // Adam creates a qualification
        let mut vc_adam_qualification = VectorClock::new("adam");
        // Jacob gets a copy and changes it
        let mut vc_jacob_qualification = vc_adam_qualification.fork("jacob").unwrap();
        // Brit gets a copy of adam and changes it
        let mut vc_brit_qualification = vc_adam_qualification.fork("brit").unwrap();
        // Nick gets a copy of brits and changes it
        let vc_nick_qualification = vc_brit_qualification.fork("nick").unwrap();

        // Jacob incorporates Nick's work
        vc_jacob_qualification
            .merge(&vc_nick_qualification)
            .unwrap();
        assert!(vc_jacob_qualification.clock_entries.get("jacob").is_some());
        assert!(vc_jacob_qualification.clock_entries.get("adam").is_some());
        assert!(vc_jacob_qualification.clock_entries.get("brit").is_some());
        assert!(vc_jacob_qualification.clock_entries.get("nick").is_some());

        vc_brit_qualification.inc().unwrap();
        let vc_brit_clock = vc_brit_qualification
            .clock_entries
            .get("brit")
            .unwrap()
            .clone();
        vc_adam_qualification.merge(&vc_brit_qualification).unwrap();
        vc_adam_qualification.merge(&vc_nick_qualification).unwrap();
        assert_eq!(
            vc_adam_qualification.clock_entries.get("brit").unwrap(),
            &vc_brit_clock
        );
    }

    // Adam creates a qualification function
    // Adam publishes that function in a module
    // Jacob installs the module
    // Jacob edits the function directly
    // Jacob shares his edited function with Adam
    // Adam accepts his edit as his new version
    // Adam publishes an update to the module
    // Jacob installs the new version of the module
    // Jacob has adam's version of his code
    #[test]
    fn share_and_merge() {
        // Adam creates a qualification function
        let mut adam_abc_func = Function::new("abc123", "adam");

        // Adam publishes that function in a module
        let mut adam_jackson5_module = Module::new("jackson5");
        adam_jackson5_module.add(adam_abc_func.clone());

        // Jacob installs the module
        let mut jacob_jackson5_module = adam_jackson5_module.clone();

        // Jacob edits the function
        let jacob_abc_func = jacob_jackson5_module.function(adam_abc_func.id()).unwrap();
        jacob_abc_func.update("easyas123", "jacob").unwrap();

        // Jacob shares his edited function with Adam, and Adam accepts it
        adam_abc_func.merge(&jacob_abc_func).unwrap();

        // Adam updates his module to the new version
        adam_jackson5_module.add(adam_abc_func.clone());

    }
}
