use crate::{
    AttributeValueId, ChangeSetId, PropId, PropertyEditorPropId, PropertyEditorValueId,
    VectorClockChangeSetId,
};

impl From<PropId> for PropertyEditorPropId {
    fn from(prop_id: PropId) -> Self {
        Self::from(::ulid::Ulid::from(prop_id))
    }
}

impl From<PropertyEditorPropId> for PropId {
    fn from(property_editor_prop_id: PropertyEditorPropId) -> Self {
        Self::from(::ulid::Ulid::from(property_editor_prop_id))
    }
}

impl From<AttributeValueId> for PropertyEditorValueId {
    fn from(id: AttributeValueId) -> Self {
        Self::from(::ulid::Ulid::from(id))
    }
}

impl From<PropertyEditorValueId> for AttributeValueId {
    fn from(id: PropertyEditorValueId) -> Self {
        Self::from(::ulid::Ulid::from(id))
    }
}

impl From<ChangeSetId> for VectorClockChangeSetId {
    fn from(id: ChangeSetId) -> Self {
        Self::from(::ulid::Ulid::from(id))
    }
}
