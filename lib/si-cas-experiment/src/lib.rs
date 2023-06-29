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
            self.counter = other.counter;
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
        forked.who = Some(who);
        forked.inc()?;
        Ok(forked)
    }

    pub fn already_seen(&self, other: &VectorClock) -> CasResult<bool> {
        let them = match &other.who {
            Some(w) => w,
            None => return Err(CasError::NoWho),
        };

        if let Some(local_view) = self.clock_entries.get(them) {
            // "Other" is newer than the last time we have seen anything from them.
            if local_view < other.clock_entries.get(them).unwrap() {
                return Ok(false);
            }
        } else {
            // We haven't seen "other" at all.
            return Ok(false);
        }

        // We've seen at least everything that other is reporting to have seen.
        Ok(true)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CompareRecommendation {
    Same,
    TakeRight,
    YouFigureItOut,
    TakeLeft,
}

#[derive(Debug, Default, Clone)]
pub struct Function {
    pub last_synced_content_hash: String,
    pub content_hash: String,
    pub vector_clock: VectorClock,
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Function {}

impl Function {
    pub fn new(content_hash: impl Into<String>, who: impl Into<String>) -> Function {
        let content_hash = content_hash.into();

        Function {
            last_synced_content_hash: content_hash.clone(),
            content_hash,
            vector_clock: VectorClock::new(who),
        }
    }

    pub fn id(&self) -> String {
        format!("{0}-{1}", self.content_hash, self.lineage_id())
    }

    pub fn lineage_id(&self) -> Ulid {
        self.vector_clock.id
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
        self.last_synced_content_hash = func.content_hash.clone();
        self.content_hash = func.content_hash.clone();
        Ok(())
    }

    pub fn receive(&self, who: impl Into<String>) -> CasResult<Function> {
        let func = Function {
            last_synced_content_hash: self.content_hash.clone(),
            content_hash: self.content_hash.clone(),
            vector_clock: self.vector_clock.fork(who)?,
        };
        Ok(func)
    }

    pub fn compare_and_recommend(&self, other: &Function) -> CasResult<CompareRecommendation> {
        // Not comparing apples to apples.
        if self.id() != other.id() {
            return Err(CasError::WrongMergeId);
        }

        // Both us and other have ended up at the same content hash, regardless of path.
        if self.content_hash == other.content_hash {
            return Ok(CompareRecommendation::Same);
        }

        // We have already seen everything in "other", and have local changes.
        if self.vector_clock.already_seen(&other.vector_clock)? {
            return Ok(CompareRecommendation::TakeLeft);
        }

        // "Other" has already seen everything we've done.
        if other.vector_clock.already_seen(&self.vector_clock)? {
            return Ok(CompareRecommendation::TakeRight);
        }

        // We haven't made any local changes, "other" has newer stuff.
        if self.content_hash == self.last_synced_content_hash {
            return Ok(CompareRecommendation::TakeRight);
        }

        // We have made changes, and "other" has newer stuff.
        Ok(CompareRecommendation::YouFigureItOut)

        //if remote's vector clock has no new additions after the one that we share
        // compare hash
        //  -- if hashes are the same, no changes, do nothing (but figure out clock?)
        //  -- if hashes are different, take yours as remote hasn't changed

        //if remote's vector clock HAS new additions after the one that we share
        //compare hash
        // -- if both hashes are different, you figure it out
        // -- if only the remote hash changed, take it (how do we know this)
        // -- if hashes are the same, do nothing (but figure out clock?)
    }
}

#[derive(Debug, Default, Clone)]
pub struct Module {
    pub content_hash: Ulid,
    pub vector_clock: VectorClock,
    pub name: String,
    pub funcs: HashMap<String, Function>,
}

impl Module {
    pub fn new(name: impl Into<String>, workspace_id: impl Into<String>) -> Module {
        let workspace_id = workspace_id.into();
        Module {
            content_hash: Ulid::new(),
            vector_clock: VectorClock::new(workspace_id),
            name: name.into(),
            funcs: HashMap::new(),
        }
    }

    pub fn id(&self) -> String {
        format!("{0}-{1}", self.content_hash, self.lineage_id())
    }

    pub fn lineage_id(&self) -> Ulid {
        self.vector_clock.id
    }

    pub fn add(&mut self, func: Function) {
        self.funcs.insert(func.id(), func);
        self.content_hash = Ulid::new();
        self.vector_clock.inc().unwrap();
    }

    pub fn remove(&mut self, func_id: impl AsRef<str>) {
        let func_id = func_id.as_ref();
        self.funcs.remove(func_id);
        self.content_hash = Ulid::new();
        self.vector_clock.inc().unwrap();
    }

    pub fn replace(&mut self, old_func_id: impl AsRef<str>, new_func: Function) {
        self.remove(old_func_id);
        self.add(new_func);
    }

    pub fn function(&mut self, function_id: impl Into<String>) -> CasResult<&mut Function> {
        let function_id = function_id.into();
        let func = self
            .funcs
            .get_mut(&function_id)
            .ok_or(CasError::NoContentHash)?;
        Ok(func)
    }
}

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: Ulid,
    pub modules: HashMap<String, Module>,
    pub funcs: HashMap<String, Function>,
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace {
            id: Ulid::new(),
            modules: HashMap::new(),
            funcs: HashMap::new(),
        }
    }

    pub fn share_module(&mut self, module_id: impl Into<String>) -> Module {
        let module_id = module_id.into();
        self.modules.get(&module_id).unwrap().clone()
    }

    pub fn install_module(&mut self, module: Module) {
        for (id, func) in module.funcs.iter() {
            self.funcs.insert(id.clone(), func.clone());
        }
        self.modules.insert(module.id().clone(), module);
    }

    pub fn create_module(&mut self, name: impl Into<String>) -> String {
        let name = name.into();
        let module = Module::new(name, self.id);
        let module_id = module.id();
        self.modules.insert(module_id.clone(), module);
        module_id
    }

    pub fn add_func_to_module(
        &mut self,
        module_id: impl Into<String>,
        func_id: impl Into<String>,
    ) -> String {
        let module_id = module_id.into();
        let func_id = func_id.into();
        let func = self.funcs.get(&func_id).unwrap().clone();
        let mut new_module = self.modules.get(&module_id).unwrap().clone();
        new_module.add(func);
        let new_module_id = new_module.id();
        self.modules.insert(new_module.id(), new_module);
        new_module_id
    }

    pub fn replace_func_in_module(
        &mut self,
        module_id: impl Into<String>,
        base_func_id: impl Into<String>,
        new_func_id: impl Into<String>,
    ) -> String {
        let module_id = module_id.into();
        let base_func_id = base_func_id.into();
        let new_func_id = new_func_id.into();
        let new_func = self.funcs.get(&new_func_id).unwrap().clone();

        let mut new_module = self.modules.get(&module_id).unwrap().clone();
        new_module.replace(&base_func_id, new_func);
        let new_module_id = new_module.id();
        self.modules.insert(new_module.id(), new_module);
        new_module_id
    }

    pub fn create_function(&mut self, content_hash: impl Into<String>) -> String {
        let content_hash = content_hash.into();
        let func = Function::new(content_hash, self.id);
        let func_id = func.id();
        self.funcs.insert(func_id.clone(), func);
        func_id
    }

    pub fn edit_function(
        &mut self,
        base_func_id: impl Into<String>,
        updated_content_hash: impl Into<String>,
    ) -> String {
        let base_func_id = base_func_id.into();
        let updated_content_hash = updated_content_hash.into();

        let mut base_func = self.funcs.get(&base_func_id).unwrap().clone();
        base_func.update(updated_content_hash, self.id).unwrap();

        let base_func_id = base_func.id().clone();
        self.funcs.insert(base_func.id(), base_func);
        base_func_id
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
        let mut adam_jackson5_module = Module::new("jackson5", "poop");
        adam_jackson5_module.add(adam_abc_func.clone());

        // Jacob installs the module
        let mut jacob_jackson5_module = adam_jackson5_module.clone();

        // Jacob edits the function
        let jacob_abc_func = jacob_jackson5_module.function(adam_abc_func.id()).unwrap();
        jacob_abc_func.update("easyas123", "jacob").unwrap();

        // Jacob shares his edited function with Adam, and Adam accepts it
        adam_abc_func.merge(jacob_abc_func).unwrap();

        // Adam updates his module to the new version
        adam_jackson5_module.add(adam_abc_func.clone());
    }

    #[test]
    fn workspace_create_func() {
        let mut workspace = Workspace::new();
        let func = workspace.create_function("parliament");
        let has_parliament = workspace
            .funcs
            .values()
            .find(|f| f.content_hash == "parliament")
            .unwrap();
        assert_eq!(has_parliament.id(), func);
    }

    #[test]
    fn workspace_create_module() {
        let mut workspace = Workspace::new();
        let funkytown_module_id = workspace.create_module("funkytown");
        assert_eq!(
            workspace.modules.get(&funkytown_module_id).unwrap().name,
            "funkytown"
        );
    }

    #[test]
    fn workspace_add_func_to_module() {
        let mut workspace = Workspace::new();
        let p_func_id = workspace.create_function("parliament");
        let funkytown_module_id = workspace.create_module("funkytown");
        let funkytown_module_id =
            workspace.add_func_to_module(funkytown_module_id.clone(), p_func_id.clone());
        workspace
            .modules
            .get(&funkytown_module_id)
            .unwrap()
            .funcs
            .get(&p_func_id)
            .unwrap();
        assert_eq!(workspace.modules.len(), 2);
    }

    #[test]
    fn workspace_share_module() {
        let mut workspace = Workspace::new();
        let p_func_id = workspace.create_function("parliament");
        let funkytown_module_id = workspace.create_module("funkytown");
        workspace.add_func_to_module(&funkytown_module_id, p_func_id);
        let shared_funkytown_module = workspace.share_module(&funkytown_module_id);
        assert_eq!(shared_funkytown_module.id(), funkytown_module_id);
    }

    #[test]
    fn workspace_install_module() {
        let mut workspace = Workspace::new();
        let p_func_id = workspace.create_function("parliament");
        let funkytown_module_id = dbg!(workspace.create_module("funkytown"));
        let funkytown_module_id =
            dbg!(workspace.add_func_to_module(&funkytown_module_id, p_func_id.clone()));
        let shared_funkytown_module = workspace.share_module(&funkytown_module_id);

        let mut other_workspace = Workspace::new();
        other_workspace.install_module(shared_funkytown_module);
        other_workspace
            .modules
            .get(&funkytown_module_id)
            .unwrap()
            .funcs
            .get(&p_func_id)
            .unwrap();
        other_workspace.funcs.get(&p_func_id).unwrap();
    }

    #[test]
    fn workspace_edit_function() {
        let mut workspace = Workspace::new();
        let p_func_id = workspace.create_function("parliament");
        let edited_p_func_id = workspace.edit_function(&p_func_id, "atomic dog");
        let p_func = workspace.funcs.get(&p_func_id).unwrap();
        assert_eq!(p_func.content_hash, "parliament");
        let edited_p_func = workspace.funcs.get(&edited_p_func_id).unwrap();
        assert_eq!(edited_p_func.content_hash, "atomic dog");
    }

    #[test]
    fn workspace_with_lots_of_modules() {
        // Jacob has a workspace
        let mut jacob_workspace = Workspace::new();

        // Jacob writes a function
        let j_func_id = jacob_workspace.create_function("parliament");

        // Jacob create a module that includes the function
        let j_module_id = jacob_workspace.create_module("parliamentary republic");
        let j_module_id = jacob_workspace.add_func_to_module(&j_module_id, &j_func_id);

        dbg!(&jacob_workspace);
        // Jacob publishes the module
        let shared_j_module = jacob_workspace.share_module(&j_module_id);

        // Brit installs Jacob's module
        let mut brit_workspace = Workspace::new();
        brit_workspace.install_module(shared_j_module.clone());

        // Nick installs Jacob's module
        let mut nick_workspace = Workspace::new();
        nick_workspace.install_module(shared_j_module.clone());

        // Nick edits the function he received from Jacob's module
        let nick_func_id = nick_workspace.edit_function(&j_func_id, "atomic dog");

        // Nick creates a module that includes his edited function
        let nick_module_id = nick_workspace.create_module("nicks parliamentary republic");
        let nick_module_id = nick_workspace.add_func_to_module(&nick_module_id, &nick_func_id);

        // Nick publishes his module
        let shared_nick_module = nick_workspace.share_module(&nick_module_id);

        // Brit installs Nick's module
        //  -> brit has two functions, one from Jacob and one from Nick
        brit_workspace.install_module(shared_nick_module);

        // Jacob updates his function
        let j_updated_func_id = jacob_workspace.edit_function(&j_func_id, "woof");
        let updated_j_module_id =
            jacob_workspace.add_func_to_module(&j_module_id, &j_updated_func_id);

        // Jacob publishes his module
        let shared_j_updated_module = jacob_workspace.share_module(&updated_j_module_id);

        // Brit installs Jacob's updated module
        brit_workspace.install_module(shared_j_updated_module);

        //  -> Brit has jacob's updated function, but nicks version remains untouched - she now has 3 total
        assert_eq!(brit_workspace.modules.len(), 3);
        assert_eq!(brit_workspace.funcs.len(), 3);
    }

    #[test]
    fn updating_a_module_with_no_workspace_changes() {
        // Adam writes a module with 3 functions
        let mut adam_workspace = Workspace::new();
        let a_func_poop_id = adam_workspace.create_function("poop");
        let a_func_canoe_id = adam_workspace.create_function("canoe");
        let a_func_paddle_id = adam_workspace.create_function("paddle");
        let a_module_id = adam_workspace.create_module("fun");
        let a_module_id = adam_workspace.add_func_to_module(&a_module_id, &a_func_poop_id);
        let a_module_id = adam_workspace.add_func_to_module(&a_module_id, &a_func_canoe_id);
        let a_module_id = adam_workspace.add_func_to_module(&a_module_id, &a_func_paddle_id);

        // Adam shares module with brit
        let a_module_shared_1 = adam_workspace.share_module(&a_module_id);

        // Brit installs Adam's updated module
        let mut brit_workspace = Workspace::new();
        brit_workspace.install_module(a_module_shared_1);

        // Adam updates one of the functions
        let a_func_rudder_id = adam_workspace.edit_function(&a_func_paddle_id, "rudder");
        let a_module_id = adam_workspace.replace_func_in_module(&a_module_id, &a_func_paddle_id, &a_func_rudder_id);

        // Adam shares updated module with brit
        let a_module_shared_2 = adam_workspace.share_module(&a_module_id);

        brit_workspace.install_module(a_module_shared_2.clone());

        // Brit has 4 functions and 2 modules
        assert_eq!(brit_workspace.modules.len(), 2);
        assert_eq!(brit_workspace.funcs.len(), 4);

        let mut nick_workspace = Workspace::new();
        nick_workspace.install_module(a_module_shared_2);
        assert_eq!(brit_workspace.modules.len(), 2);
        dbg!(&adam_workspace);
        dbg!(&nick_workspace);
        assert_eq!(nick_workspace.funcs.len(), 3);
    }

    // Adam writes a module with 3 different functions
    // Adam shares the module with brit
    // Brit installs adam's module
    // Brit edits one of adam's functions locally
    // Adam updates the same function in his workspace
    // Adam shares the updated module with brit
    // Brit gets notified there is a decision to be made

}
