use serde::{Deserialize, Serialize};

use crate::ulid::Ulid;

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct VectorClockChangeSetId(Ulid);

impl std::fmt::Display for VectorClockChangeSetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct VectorClockActorId(Ulid);

impl std::fmt::Display for VectorClockActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Serialize, Deserialize, Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct VectorClockId {
    change_set_id: VectorClockChangeSetId,
    actor_id: VectorClockActorId,
}

impl std::fmt::Display for VectorClockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(ChangeSet({}), Actor({}))",
            self.change_set_id, self.actor_id
        )
    }
}

impl VectorClockId {
    pub fn new(change_set_id: VectorClockChangeSetId, actor_id: VectorClockActorId) -> Self {
        Self {
            change_set_id,
            actor_id,
        }
    }

    pub fn change_set_id(&self) -> VectorClockChangeSetId {
        self.change_set_id
    }

    pub fn actor_id(&self) -> VectorClockActorId {
        self.actor_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() {
        let cs_text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
        let actor_text = "00000000000000000000000000";

        let change_set_ulid = Ulid::from_string(cs_text).expect("make ulid from string");
        let actor_ulid = Ulid::from_string(actor_text).expect("make actor from string");

        let vector_clock_id = VectorClockId::new(
            VectorClockChangeSetId(change_set_ulid),
            VectorClockActorId(actor_ulid),
        );

        let expected = format!("(ChangeSet({cs_text}), Actor({actor_text}))");
        let display = format!("{vector_clock_id}");
        assert_eq!(expected, display);
    }
}
