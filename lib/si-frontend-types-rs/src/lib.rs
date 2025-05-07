pub mod action;
mod approval_requirement;
mod audit_log;
mod change_set;
pub mod checksum;
mod component;
mod conflict;
pub mod fs;
mod func;
pub mod index;
pub mod materialized_view;
mod module;
pub mod newhotness;
pub mod object;
pub mod reference;
pub mod schema_variant;
pub mod view;
mod workspace;

pub use crate::{
    approval_requirement::ApprovalRequirementDefinition,
    audit_log::AuditLog,
    change_set::{
        ChangeSet,
        ChangeSetApproval,
        ChangeSetApprovalRequirement,
        ChangeSetApprovals,
        CreateChangeSetRequest,
        CreateChangeSetResponse,
    },
    component::{
        ChangeStatus,
        ConnectionAnnotation,
        DiagramComponentView,
        DiagramSocket,
        DiagramSocketDirection,
        DiagramSocketNodeSide,
        GeometryAndView,
        GridPoint,
        PotentialConnection,
        PotentialMatch,
        RawGeometry,
        Size2D,
        StringGeometry,
    },
    conflict::ConflictWithHead,
    func::{
        AttributeArgumentBinding,
        FuncArgument,
        FuncArgumentKind,
        FuncBinding,
        FuncBindings,
        FuncCode,
        FuncKind,
        FuncSummary,
        LeafInputLocation,
    },
    materialized_view::MaterializedView,
    module::{
        BuiltinModules,
        LatestModule,
        ModuleContributeRequest,
        ModuleDetails,
        ModuleSummary,
        SyncedModules,
    },
    schema_variant::{
        ComponentType,
        InputSocket,
        ListVariantsResponse,
        OutputSocket,
        Prop,
        PropKind,
        SchemaVariant,
        UninstalledVariant,
    },
    view::{
        View,
        ViewList,
    },
    workspace::WorkspaceMetadata,
};

#[cfg(test)]
mod tests {
    use serde::{
        Deserialize,
        Serialize,
    };

    use crate::checksum::FrontendChecksum;

    #[test]
    fn enum_with_tuple_variant_bytestreams() {
        #[derive(
            Clone,
            Debug,
            Deserialize,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Serialize,
            si_frontend_types_macros::FrontendChecksum,
        )]
        #[remain::sorted]
        #[serde(rename_all = "camelCase")]
        enum Todd {
            Howard(Vec<u8>, Vec<u8>),
        }

        let item_one = "oblivion remastered";
        let item_two = "tes vi";

        let todd_one = {
            let mut first_vec = Vec::new();
            first_vec.extend(item_one.as_bytes());
            first_vec.extend(item_two.as_bytes());
            let second_vec = Vec::new();
            let todd = Todd::Howard(first_vec, second_vec);
            FrontendChecksum::checksum(&todd)
        };

        let todd_two = {
            let mut first_vec = Vec::new();
            first_vec.extend(item_one.as_bytes());
            let mut second_vec = Vec::new();
            second_vec.extend(item_two.as_bytes());
            let todd = Todd::Howard(first_vec, second_vec);
            FrontendChecksum::checksum(&todd)
        };

        let todd_three = {
            let first_vec = Vec::new();
            let mut second_vec = Vec::new();
            second_vec.extend(item_one.as_bytes());
            second_vec.extend(item_two.as_bytes());
            let todd = Todd::Howard(first_vec, second_vec);
            FrontendChecksum::checksum(&todd)
        };

        assert_ne!(todd_one, todd_two);
        assert_ne!(todd_one, todd_three);
        assert_ne!(todd_two, todd_three);
    }

    #[test]
    fn struct_with_bytestream_fields() {
        #[derive(
            Clone,
            Debug,
            Deserialize,
            Eq,
            PartialEq,
            Ord,
            PartialOrd,
            Serialize,
            si_frontend_types_macros::FrontendChecksum,
        )]
        #[remain::sorted]
        #[serde(rename_all = "camelCase")]
        struct Example {
            howard: Vec<u8>,
            todd: Vec<u8>,
        }

        let item_one = "oblivion remastered";
        let item_two = "tes vi";

        let example_one = {
            let mut howard = Vec::new();
            howard.extend(item_one.as_bytes());
            howard.extend(item_two.as_bytes());
            let todd = Vec::new();
            let example = Example { howard, todd };
            FrontendChecksum::checksum(&example)
        };

        let example_two = {
            let mut howard = Vec::new();
            howard.extend(item_one.as_bytes());
            let mut todd = Vec::new();
            todd.extend(item_two.as_bytes());
            let example = Example { howard, todd };
            FrontendChecksum::checksum(&example)
        };

        let example_three = {
            let howard = Vec::new();
            let mut todd = Vec::new();
            todd.extend(item_one.as_bytes());
            todd.extend(item_two.as_bytes());
            let example = Example { howard, todd };
            FrontendChecksum::checksum(&example)
        };

        assert_ne!(example_one, example_two);
        assert_ne!(example_one, example_three);
        assert_ne!(example_two, example_three);
    }
}
