use crate::ulid::Ulid;

pub use si_id::VectorClockActorId;
pub use si_id::VectorClockChangeSetId;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct VectorClockId {
    change_set_id: VectorClockChangeSetId,
    actor_id: VectorClockActorId,
}

pub struct VectorClockIdStringDeserializeVisitor;

impl<'de> serde::de::Visitor<'de> for VectorClockIdStringDeserializeVisitor {
    type Value = VectorClockId;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a string reprensenting a VectorClockId")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let (change_set_id_string, actor_id_string) = v.split_once(';').ok_or(E::custom(
            format!("{v} is not a valid VectorClockId string representation."),
        ))?;

        let change_set_id = Ulid::from_string(change_set_id_string).map_err(|e| {
            E::custom(format!(
                "VectorClock ChangeSetId \"{change_set_id_string}\" is not a valid Ulid representation: {e}"
            ))
        })?;
        let actor_id = Ulid::from_string(actor_id_string).map_err(|e| {
            E::custom(format!(
                "VectorClock ActorId \"{actor_id_string}\" is not a valid Ulid representation: {e}"
            ))
        })?;

        Ok(VectorClockId::new(change_set_id, actor_id))
    }
}

impl<'de> serde::Deserialize<'de> for VectorClockId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(VectorClockIdStringDeserializeVisitor)
    }
}

impl serde::Serialize for VectorClockId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let string_vector_clock_id = format!("{};{}", self.change_set_id, self.actor_id);
        serializer.serialize_str(&string_vector_clock_id)
    }
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
    pub fn new(
        change_set_id: impl Into<VectorClockChangeSetId>,
        actor_id: impl Into<VectorClockActorId>,
    ) -> Self {
        Self {
            change_set_id: change_set_id.into(),
            actor_id: actor_id.into(),
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
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn display() {
        let cs_text = "01D39ZY06FGSCTVN4T2V9PKHFZ";
        let actor_text = "00000000000000000000000000";

        let change_set_ulid = Ulid::from_string(cs_text).expect("make ulid from string");
        let actor_ulid = Ulid::from_string(actor_text).expect("make actor from string");

        let vector_clock_id = VectorClockId::new(
            VectorClockChangeSetId::from(change_set_ulid),
            VectorClockActorId::from(actor_ulid),
        );

        let expected = format!("(ChangeSet({cs_text}), Actor({actor_text}))");
        let display = format!("{vector_clock_id}");
        assert_eq!(expected, display);
    }

    #[test]
    fn serde_json() {
        let mut vector_clock_map = HashMap::new();

        for i in 0..10 {
            let vector_clock_id = VectorClockId::new(Ulid::new(), Ulid::new());
            vector_clock_map.insert(vector_clock_id, i);
        }

        let json_string =
            serde_json::to_string(&vector_clock_map).expect("should serialize to string");
        let deserialized: HashMap<VectorClockId, i32> =
            serde_json::from_str(&json_string).expect("should deserialize");

        assert_eq!(vector_clock_map, deserialized);
    }
}
