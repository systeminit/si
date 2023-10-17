


use crate::change_set_pointer::ChangeSetPointer;

use crate::property_editor::schema::WidgetKind;
use crate::schema::variant::root_prop::RootProp;
use crate::validation::Validation;
use crate::workspace_snapshot::api::prop::PropParent;
use crate::workspace_snapshot::WorkspaceSnapshotResult;
use crate::{
    schema::variant::leaves::LeafKind, DalContext, PropId, PropKind, SchemaId, SchemaVariantId,
    StandardModel, WorkspaceSnapshot,
};

impl WorkspaceSnapshot {
    /// Create and set a [`RootProp`] for the [`SchemaVariant`].
    pub async fn schema_variant_create_root_prop_tree(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        schema_variant_id: SchemaVariantId,
        _schema_id: SchemaId,
    ) -> WorkspaceSnapshotResult<RootProp> {
        let root_prop = self
            .prop_create(
                ctx,
                change_set,
                "root",
                PropKind::Object,
                None,
                PropParent::SchemaVariant(schema_variant_id),
                true,
            )
            .await?;

        let si_prop_id = self
            .schema_variant_root_prop_setup_si(ctx, change_set, root_prop.id())
            .await?;

        let domain_prop = self
            .prop_create(
                ctx,
                change_set,
                "domain",
                PropKind::Object,
                None,
                PropParent::OrderedProp(root_prop.id()),
                true,
            )
            .await?;

        let secrets_prop = self
            .prop_create(
                ctx,
                change_set,
                "secrets",
                PropKind::Object,
                None,
                PropParent::OrderedProp(root_prop.id()),
                true,
            )
            .await?;

        let resource_prop_id = self
            .schema_variant_root_prop_setup_resource(
                ctx,
                change_set,
                root_prop.id(),
                schema_variant_id,
            )
            .await?;

        let resource_value_prop_id = self
            .schema_variant_root_prop_setup_resource_value(
                ctx,
                change_set,
                root_prop.id(),
                schema_variant_id,
            )
            .await?;

        let code_prop_id = self
            .schema_variant_root_prop_setup_code(ctx, change_set, root_prop.id(), schema_variant_id)
            .await?;
        let qualification_prop_id = self
            .schema_variant_root_prop_setup_qualification(
                ctx,
                change_set,
                root_prop.id(),
                schema_variant_id,
            )
            .await?;

        let deleted_at_prop = self
            .prop_create(
                ctx,
                change_set,
                "deleted_at",
                PropKind::String,
                None,
                PropParent::OrderedProp(root_prop.id()),
                false,
            )
            .await?;
        self.prop_modify_by_id(ctx, change_set, deleted_at_prop.id(), |deleted_at_prop| {
            deleted_at_prop.hidden = true;
            Ok(())
        })
        .await?;

        // Now that the structure is set up, we can populate default
        // AttributePrototypes to be updated appropriately below.
        self.schema_variant_create_default_prototypes(ctx, change_set, schema_variant_id)
            .await?;

        self.schema_variant_create_implicit_internal_providers(ctx, change_set, schema_variant_id)
            .await?;

        Ok(RootProp {
            prop_id: root_prop.id(),
            si_prop_id,
            domain_prop_id: domain_prop.id(),
            resource_value_prop_id,
            resource_prop_id,
            secrets_prop_id: secrets_prop.id(),
            code_prop_id,
            qualification_prop_id,
            deleted_at_prop_id: deleted_at_prop.id(),
        })
    }

    async fn schema_variant_root_prop_insert_leaf_props(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        leaf_kind: LeafKind,
        root_prop_id: PropId,
        _schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<(PropId, PropId)> {
        let (leaf_prop_name, leaf_item_prop_name) = leaf_kind.prop_names();

        let leaf_prop = self
            .prop_create(
                ctx,
                change_set,
                leaf_prop_name,
                PropKind::Map,
                None,
                PropParent::OrderedProp(root_prop_id),
                true,
            )
            .await?;
        self.prop_modify_by_id(ctx, change_set, leaf_prop.id(), |leaf_prop| {
            leaf_prop.hidden = true;
            Ok(())
        })
        .await?;

        let leaf_item_prop = self
            .prop_create(
                ctx,
                change_set,
                leaf_item_prop_name,
                PropKind::Object,
                None,
                PropParent::OrderedProp(leaf_prop.id()),
                true,
            )
            .await?;
        self.prop_modify_by_id(ctx, change_set, leaf_item_prop.id(), |leaf_item_prop| {
            leaf_item_prop.hidden = true;
            Ok(())
        })
        .await?;

        Ok((leaf_prop.id(), leaf_item_prop.id()))
    }

    async fn schema_variant_root_prop_setup_si(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        root_prop_id: PropId,
    ) -> WorkspaceSnapshotResult<PropId> {
        let si_prop = self
            .prop_create(
                ctx,
                change_set,
                "si",
                PropKind::Object,
                None,
                PropParent::OrderedProp(root_prop_id),
                true,
            )
            .await?;

        let _si_name_prop = self
            .prop_create(
                ctx,
                change_set,
                "name",
                PropKind::String,
                None,
                PropParent::OrderedProp(si_prop.id()),
                false,
            )
            .await?;

        // The protected prop ensures a component cannot be deleted in the configuration diagram.
        let _protected_prop = self
            .prop_create(
                ctx,
                change_set,
                "protected",
                PropKind::Boolean,
                None,
                PropParent::OrderedProp(si_prop.id()),
                false,
            )
            .await?;

        // The type prop controls the type of the configuration node. The default type can be
        // determined by the schema variant author. The widget options correspond to the component
        // type enumeration.
        let _type_prop = self
            .prop_create(
                ctx,
                change_set,
                "type",
                PropKind::String,
                Some((
                    WidgetKind::Select,
                    Some(serde_json::json!([
                        {
                            "label": "Component",
                            "value": "component",
                        },
                        {
                            "label": "Configuration Frame",
                            "value": "configurationFrame",
                        },
                        {
                            "label": "Aggregation Frame",
                            "value": "aggregationFrame",
                        },
                    ])),
                )),
                PropParent::OrderedProp(si_prop.id()),
                false,
            )
            .await?;

        // Override the schema variant color for nodes on the diagram.
        let color_prop = self
            .prop_create(
                ctx,
                change_set,
                "color",
                PropKind::String,
                Some((WidgetKind::Color, None)),
                PropParent::OrderedProp(si_prop.id()),
                false,
            )
            .await?;

        self.validation_prototype_create_in_memory(
            ctx,
            change_set,
            Validation::StringIsHexColor { value: None },
            color_prop.id(),
        )
        .await?;

        Ok(si_prop.id())
    }

    async fn schema_variant_root_prop_setup_resource_value(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        root_prop_id: PropId,
        _schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<PropId> {
        let resource_value_prop = self
            .prop_create(
                ctx,
                change_set,
                "resource_value",
                PropKind::Object,
                None,
                PropParent::OrderedProp(root_prop_id),
                true,
            )
            .await?;
        self.prop_modify_by_id(
            ctx,
            change_set,
            resource_value_prop.id(),
            |resource_value_prop| {
                resource_value_prop.hidden = true;
                Ok(())
            },
        )
        .await?;

        Ok(resource_value_prop.id())
    }

    async fn schema_variant_root_prop_setup_resource(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        root_prop_id: PropId,
        _schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<PropId> {
        let resource_prop = self
            .prop_create(
                ctx,
                change_set,
                "resource",
                PropKind::Object,
                None,
                PropParent::OrderedProp(root_prop_id),
                false,
            )
            .await?;
        self.prop_modify_by_id(ctx, change_set, resource_prop.id(), |resource_prop| {
            resource_prop.hidden = true;
            Ok(())
        })
        .await?;

        let resource_status_prop = self
            .prop_create(
                ctx,
                change_set,
                "status",
                PropKind::String,
                None,
                PropParent::Prop(root_prop_id),
                false,
            )
            .await?;
        self.prop_modify_by_id(
            ctx,
            change_set,
            resource_status_prop.id(),
            |resource_status_prop| {
                resource_status_prop.hidden = true;
                Ok(())
            },
        )
        .await?;

        let resource_message_prop = self
            .prop_create(
                ctx,
                change_set,
                "message",
                PropKind::String,
                None,
                PropParent::Prop(root_prop_id),
                false,
            )
            .await?;
        self.prop_modify_by_id(
            ctx,
            change_set,
            resource_message_prop.id(),
            |resource_message_prop| {
                resource_message_prop.hidden = true;
                Ok(())
            },
        )
        .await?;

        let resource_logs_prop = self
            .prop_create(
                ctx,
                change_set,
                "logs",
                PropKind::Array,
                None,
                PropParent::Prop(root_prop_id),
                true,
            )
            .await?;
        self.prop_modify_by_id(
            ctx,
            change_set,
            resource_logs_prop.id(),
            |resource_logs_prop| {
                resource_logs_prop.hidden = true;
                Ok(())
            },
        )
        .await?;

        let resource_logs_log_prop = self
            .prop_create(
                ctx,
                change_set,
                "log",
                PropKind::String,
                None,
                PropParent::OrderedProp(root_prop_id),
                false,
            )
            .await?;
        self.prop_modify_by_id(
            ctx,
            change_set,
            resource_logs_log_prop.id(),
            |resource_logs_log_prop| {
                resource_logs_log_prop.hidden = true;
                Ok(())
            },
        )
        .await?;

        let resource_payload_prop = self
            .prop_create(
                ctx,
                change_set,
                "payload",
                PropKind::String,
                None,
                PropParent::Prop(root_prop_id),
                false,
            )
            .await?;
        self.prop_modify_by_id(
            ctx,
            change_set,
            resource_payload_prop.id(),
            |resource_payload_prop| {
                resource_payload_prop.hidden = true;
                Ok(())
            },
        )
        .await?;

        let resource_last_synced_prop = self
            .prop_create(
                ctx,
                change_set,
                "resource_last_synced_prop",
                PropKind::String,
                None,
                PropParent::Prop(root_prop_id),
                false,
            )
            .await?;
        self.prop_modify_by_id(
            ctx,
            change_set,
            resource_last_synced_prop.id(),
            |resource_last_synced_prop| {
                resource_last_synced_prop.hidden = true;
                Ok(())
            },
        )
        .await?;

        Ok(resource_prop.id())
    }

    async fn schema_variant_root_prop_setup_code(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        root_prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<PropId> {
        let (code_map_prop_id, code_map_item_prop_id) = self
            .schema_variant_root_prop_insert_leaf_props(
                ctx,
                change_set,
                LeafKind::CodeGeneration,
                root_prop_id,
                schema_variant_id,
            )
            .await?;

        let child_code_prop = self
            .prop_create(
                ctx,
                change_set,
                "code",
                PropKind::String,
                None,
                PropParent::OrderedProp(code_map_item_prop_id),
                false,
            )
            .await?;
        self.prop_modify_by_id(ctx, change_set, child_code_prop.id(), |child_code_prop| {
            child_code_prop.hidden = true;
            Ok(())
        })
        .await?;

        let child_format_prop = self
            .prop_create(
                ctx,
                change_set,
                "format",
                PropKind::String,
                None,
                PropParent::OrderedProp(code_map_item_prop_id),
                false,
            )
            .await?;
        self.prop_modify_by_id(
            ctx,
            change_set,
            child_format_prop.id(),
            |child_format_prop| {
                child_format_prop.hidden = true;
                Ok(())
            },
        )
        .await?;

        Ok(code_map_prop_id)
    }

    async fn schema_variant_root_prop_setup_qualification(
        &mut self,
        ctx: &DalContext,
        change_set: &ChangeSetPointer,
        root_prop_id: PropId,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotResult<PropId> {
        let (qualification_map_prop_id, qualification_map_item_prop_id) = self
            .schema_variant_root_prop_insert_leaf_props(
                ctx,
                change_set,
                LeafKind::Qualification,
                root_prop_id,
                schema_variant_id,
            )
            .await?;

        let child_qualified_prop = self
            .prop_create(
                ctx,
                change_set,
                "result",
                PropKind::String,
                None,
                PropParent::OrderedProp(qualification_map_item_prop_id),
                false,
            )
            .await?;
        self.prop_modify_by_id(
            ctx,
            change_set,
            child_qualified_prop.id(),
            |child_qualified_prop| {
                child_qualified_prop.hidden = true;
                Ok(())
            },
        )
        .await?;

        let child_message_prop = self
            .prop_create(
                ctx,
                change_set,
                "message",
                PropKind::String,
                None,
                PropParent::OrderedProp(qualification_map_item_prop_id),
                false,
            )
            .await?;
        self.prop_modify_by_id(
            ctx,
            change_set,
            child_message_prop.id(),
            |child_message_prop| {
                child_message_prop.hidden = true;
                Ok(())
            },
        )
        .await?;

        Ok(qualification_map_prop_id)
    }
}
