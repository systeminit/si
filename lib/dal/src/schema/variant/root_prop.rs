//! This module contains (and is oriented around) the [`RootProp`]. This object is not persisted
//! to the database.

use strum::{
    AsRefStr,
    Display as EnumDisplay,
    EnumIter,
    EnumString,
};

use crate::{
    DalContext,
    Prop,
    PropId,
    PropKind,
    SchemaVariant,
    SchemaVariantId,
    prop::PropPath,
    property_editor::schema::WidgetKind,
    schema::variant::{
        SchemaVariantResult,
        leaves::LeafKind,
    },
};

pub mod component_type;

/// This enum contains the subtree names for every direct child [`Prop`](crate::Prop) of
/// [`RootProp`](RootProp). Not all children will be of the same [`PropKind`](crate::PropKind).
#[remain::sorted]
#[derive(AsRefStr, EnumIter, EnumString, EnumDisplay, Debug)]
pub enum RootPropChild {
    /// Corresponds to the "/root/code" subtree.
    Code,
    /// Corresponds to the "/root/deleted_at" subtree.
    DeletedAt,
    /// Corresponds to the "/root/domain" subtree.
    Domain,
    /// Corresponds to the "/root/qualification" subtree.
    Qualification,
    /// Corresponds to the "/root/resource" subtree.
    Resource,
    /// Corresponds to the "/root/secrets" subtree.
    Secrets,
    /// Corresponds to the "/root/si" subtree.
    Si,
}

impl RootPropChild {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Si => "si",
            Self::Domain => "domain",
            Self::Resource => "resource",
            Self::Code => "code",
            Self::Qualification => "qualification",
            Self::DeletedAt => "deleted_at",
            Self::Secrets => "secrets",
        }
    }

    pub fn prop_path(&self) -> PropPath {
        PropPath::new(["root", self.as_str()])
    }
}

/// This enum contains the subtree names for every direct child [`Prop`](crate::Prop) of "/root/si".
/// These [`Props`](crate::Prop) are available for _every_ [`SchemaVariant`](crate::SchemaVariant).
#[remain::sorted]
#[derive(Debug)]
pub enum SiPropChild {
    /// Corresponds to the "/root/si/Color" [`Prop`](crate::Prop).
    Color,
    /// Corresponds to the "/root/si/name" [`Prop`](crate::Prop).
    Name,
    /// Corresponds to the "/root/si/protected" [`Prop`](crate::Prop).
    Protected,
    /// Corresponds to the "/root/si/type" [`Prop`](crate::Prop).
    Type,
}

impl SiPropChild {
    /// Return the _case-sensitive_ name for the corresponding [`Prop`](crate::Prop).
    pub fn prop_name(&self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Protected => "protected",
            Self::Type => "type",
            Self::Color => "color",
        }
    }
}

/// Contains the root [`PropId`](crate::Prop) and its immediate children for a
/// [`SchemaVariant`](crate::SchemaVariant). These [`Props`](crate::Prop) are also those that
/// correspond to the "root" [`Props`](crate::Prop) on the [`ComponentView`](crate::ComponentView)
/// "properties" field.
#[derive(Debug, Copy, Clone)]
pub struct RootProp {
    /// The parent of the other [`Props`](crate::Prop) on [`self`](Self).
    pub prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to System Initiative metadata.
    pub si_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to the real world _model_.
    pub domain_prop_id: PropId,
    /// The parent of the resource [`Props`](crate::Prop) corresponding to the real world _resource_.
    pub resource_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) that are of secret value.
    pub secrets_prop_id: PropId,
    /// All information needed to populate the _model_ should be derived from this tree.
    pub resource_value_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to code generation
    /// [`Funcs`](crate::Func).
    pub code_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to qualification
    /// [`Funcs`](crate::Func).
    pub qualification_prop_id: PropId,
    /// The deleted_at prop on [`self`](Self).
    pub deleted_at_prop_id: PropId,
}

impl RootProp {
    /// Create and set a [`RootProp`] for the [`SchemaVariant`].
    /// *BIG BOLD BANNER*: changes to this structure must be followed up with
    /// changes to the structure of the root props defined in the si-pkg crate!
    /// TODO: use the si-pkg definition of the root prop to drive this code, so
    /// we have one source of truth
    pub async fn new(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Self> {
        let root_prop = Prop::new_root(
            ctx,
            "root",
            PropKind::Object,
            false,
            None,
            None,
            None,
            None,
            Default::default(),
            schema_variant_id,
        )
        .await?;
        let root_prop_id = root_prop.id();

        let si_prop_id = Self::setup_si(ctx, root_prop_id).await?;

        let domain_prop =
            Prop::new_without_ui_optionals(ctx, "domain", PropKind::Object, root_prop_id).await?;

        let secrets_prop =
            Prop::new_without_ui_optionals(ctx, "secrets", PropKind::Object, root_prop_id).await?;

        // info!("setting up resource");
        let resource_prop_id = Self::setup_resource(ctx, root_prop_id).await?;
        // info!("setting up resource value");
        let resource_value_prop_id = Self::setup_resource_value(ctx, root_prop_id).await?;
        // info!("setting up code");
        let code_prop_id = Self::setup_code(ctx, root_prop_id).await?;
        // info!("setting up qualification");
        let qualification_prop_id = Self::setup_qualification(ctx, root_prop_id).await?;

        // info!("setting up deleted at");
        let deleted_at_prop = Prop::new(
            ctx,
            "deleted_at",
            PropKind::String,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            root_prop_id,
        )
        .await?;

        // Now that the structure is set up, we can populate default
        // AttributePrototypes to be updated appropriately below.
        SchemaVariant::create_default_prototypes(ctx, schema_variant_id).await?;
        //SchemaVariant::mark_props_as_able_to_be_used_as_prototype_args(ctx, schema_variant_id)?;

        Ok(RootProp {
            prop_id: root_prop_id,
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

    async fn insert_leaf_props(
        ctx: &DalContext,
        leaf_kind: LeafKind,
        root_prop_id: PropId,
    ) -> SchemaVariantResult<(PropId, PropId)> {
        let (leaf_prop_name, leaf_item_prop_name) = leaf_kind.prop_names();

        let leaf_prop = Prop::new(
            ctx,
            leaf_prop_name,
            PropKind::Map,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            root_prop_id,
        )
        .await?;

        let leaf_item_prop = Prop::new(
            ctx,
            leaf_item_prop_name,
            PropKind::Object,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            leaf_prop.id(),
        )
        .await?;

        Ok((leaf_prop.id(), leaf_item_prop.id()))
    }

    async fn setup_si(ctx: &DalContext, root_prop_id: PropId) -> SchemaVariantResult<PropId> {
        let si_prop =
            Prop::new_without_ui_optionals(ctx, "si", PropKind::Object, root_prop_id).await?;

        let _si_name_prop =
            Prop::new_without_ui_optionals(ctx, "name", PropKind::String, si_prop.id()).await?;

        // The protected prop ensures a component cannot be deleted in the configuration diagram.
        let _protected_prop =
            Prop::new_without_ui_optionals(ctx, "protected", PropKind::Boolean, si_prop.id())
                .await?;

        // The type prop controls the type of the configuration node. The default type can be
        // determined by the schema variant author. The widget options correspond to the component
        // type enumeration.
        Prop::new(
            ctx,
            "type",
            PropKind::String,
            false,
            None,
            None,
            Some((
                WidgetKind::Select,
                Some(serde_json::json!([
                    {
                        "label": "Component",
                        "value": "component",
                    },
                    {
                        "label": "Configuration Frame (down)",
                        "value": "configurationFrameDown",
                    },
                    {
                        "label": "Configuration Frame (up)",
                        "value": "configurationFrameUp",
                    },
                    {
                        "label": "Aggregation Frame",
                        "value": "aggregationFrame",
                    },
                ])),
            )),
            None,
            Default::default(),
            si_prop.id(),
        )
        .await?;

        // Override the schema variant color for nodes on the diagram.
        Prop::new(
            ctx,
            "color",
            PropKind::String,
            false,
            None,
            None,
            Some((WidgetKind::Color, None)),
            None,
            Default::default(),
            si_prop.id(),
        )
        .await?;

        // The `resourceId` is a prop that uniquely identifies the resource,
        // useful for importing an existing resource from reality into the model
        Prop::new_without_ui_optionals(ctx, "resourceId", PropKind::String, si_prop.id()).await?;

        Ok(si_prop.id())
    }

    async fn setup_resource(ctx: &DalContext, root_prop_id: PropId) -> SchemaVariantResult<PropId> {
        // /root/resource
        let resource_prop = Prop::new(
            ctx,
            "resource",
            PropKind::Object,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            root_prop_id,
        )
        .await?;

        // /root/resource/status
        let _resource_status_prop = Prop::new(
            ctx,
            "status",
            PropKind::String,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            resource_prop.id(),
        )
        .await?;

        // /root/resource/message
        let _resource_message_prop = Prop::new(
            ctx,
            "message",
            PropKind::String,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            resource_prop.id(),
        )
        .await?;

        // /root/resource/payload
        let _resource_payload_prop = Prop::new(
            ctx,
            "payload",
            PropKind::Json,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            resource_prop.id(),
        )
        .await?;

        // /root/resource/last_synced
        let _resource_last_synced_prop = Prop::new(
            ctx,
            "last_synced",
            PropKind::String,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            resource_prop.id(),
        )
        .await?;

        Ok(resource_prop.id())
    }

    async fn setup_resource_value(
        ctx: &DalContext,
        root_prop_id: PropId,
    ) -> SchemaVariantResult<PropId> {
        let resource_value_prop = Prop::new(
            ctx,
            "resource_value",
            PropKind::Object,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            root_prop_id,
        )
        .await?;

        Ok(resource_value_prop.id())
    }

    async fn setup_code(ctx: &DalContext, root_prop_id: PropId) -> SchemaVariantResult<PropId> {
        let (code_map_prop_id, code_map_item_prop_id) =
            Self::insert_leaf_props(ctx, LeafKind::CodeGeneration, root_prop_id).await?;

        let _child_code_prop = Prop::new(
            ctx,
            "code",
            PropKind::String,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            code_map_item_prop_id,
        )
        .await?;

        let _child_format_prop = Prop::new(
            ctx,
            "format",
            PropKind::String,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            code_map_item_prop_id,
        )
        .await?;

        Ok(code_map_prop_id)
    }

    async fn setup_qualification(
        ctx: &DalContext,
        root_prop_id: PropId,
    ) -> SchemaVariantResult<PropId> {
        let (qualification_map_prop_id, qualification_map_item_prop_id) =
            Self::insert_leaf_props(ctx, LeafKind::Qualification, root_prop_id).await?;

        let _child_qualified_prop = Prop::new(
            ctx,
            "result",
            PropKind::String,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            qualification_map_item_prop_id,
        )
        .await?;

        let _child_message_prop = Prop::new(
            ctx,
            "message",
            PropKind::String,
            true,
            None,
            None,
            None,
            None,
            Default::default(),
            qualification_map_item_prop_id,
        )
        .await?;

        Ok(qualification_map_prop_id)
    }
}
