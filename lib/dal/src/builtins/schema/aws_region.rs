use serde::{Deserialize, Serialize};

use crate::builtins::schema::MigrationDriver;
use crate::builtins::BuiltinsError;
use crate::component::ComponentKind;
use crate::edit_field::widget::WidgetKind;
use crate::property_editor::SelectWidgetOption;
use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::socket::SocketArity;
use crate::validation::Validation;
use crate::AttributeValueError;
use crate::{action_prototype::ActionKind, ComponentType};
use crate::{
    func::argument::FuncArgument, ActionPrototype, ActionPrototypeContext,
    AttributePrototypeArgument, AttributeReadContext, AttributeValue, BuiltinsResult, DalContext,
    ExternalProvider, Func, InternalProvider, PropKind, SchemaError, StandardModel,
    WorkflowPrototype, WorkflowPrototypeContext,
};

// Documentation URL(s)
const AWS_REGIONS_DOCS_URL: &str =
    "https://docs.aws.amazon.com/general/latest/gr/rande.html#region-names-codes";

// Dataset(s)
const REGIONS_JSON_STR: &str = include_str!("data/aws_regions.json");

/// Used for deserializing the contents of the regions dataset.
#[derive(Deserialize, Serialize, Debug)]
pub struct AwsRegion {
    name: String,
    code: String,
}

impl From<&AwsRegion> for SelectWidgetOption {
    fn from(region: &AwsRegion) -> Self {
        Self {
            label: format!("{} - {}", region.code, region.name),
            value: region.code.clone(),
        }
    }
}

impl MigrationDriver {
    /// A [`Schema`](crate::Schema) migration for [`AWS Region`](https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/using-regions-availability-zones.html).
    pub async fn migrate_aws_region(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant(
                ctx,
                SchemaVariantDefinitionMetadataJson::new(
                    "Region",
                    None::<&str>,
                    ui_menu_category,
                    node_color,
                    ComponentKind::Standard,
                    None,
                ),
                None,
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        // Prop Creation
        let regions_json: Vec<AwsRegion> = serde_json::from_str(REGIONS_JSON_STR)?;
        let regions_dropdown_options = regions_json
            .iter()
            .map(SelectWidgetOption::from)
            .collect::<Vec<SelectWidgetOption>>();
        let regions_dropdown_options_json = serde_json::to_value(regions_dropdown_options)?;

        let region_prop = self
            .create_prop(
                ctx,
                "region",
                PropKind::String,
                Some((WidgetKind::ComboBox, Some(regions_dropdown_options_json))),
                Some(root_prop.domain_prop_id),
                Some(AWS_REGIONS_DOCS_URL.to_string()),
            )
            .await?;

        // Validation Creation
        let expected = regions_json
            .iter()
            .map(|r| r.code.clone())
            .collect::<Vec<String>>();
        self.create_validation(
            ctx,
            Validation::StringInStringArray {
                value: None,
                expected,
                display_expected: true,
            },
            *region_prop.id(),
            *schema.id(),
            *schema_variant.id(),
        )
        .await?;

        // Output Socket
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;
        let (region_external_provider, _output_socket) = ExternalProvider::new_with_socket(
            ctx,
            *schema.id(),
            *schema_variant.id(),
            "Region",
            None,
            identity_func_item.func_id,
            identity_func_item.func_binding_id,
            identity_func_item.func_binding_return_value_id,
            SocketArity::Many,
            false,
        )
        .await?;

        // Wrap it up.
        schema_variant
            .finalize(ctx, Some(ComponentType::ConfigurationFrame))
            .await?;

        let region_implicit_internal_provider =
            InternalProvider::find_for_prop(ctx, *region_prop.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::ImplicitInternalProviderNotFoundForProp(*region_prop.id())
                })?;

        // domain/region to si/name
        let si_name_prop = self
            .find_child_prop_by_name(ctx, root_prop.si_prop_id, "name")
            .await?;

        self.set_default_value_for_prop(ctx, *si_name_prop.id(), serde_json::json!["region"])
            .await?;

        let si_name_attribute_value = AttributeValue::find_for_context(
            ctx,
            AttributeReadContext::default_with_prop(*si_name_prop.id()),
        )
        .await?
        .ok_or(AttributeValueError::Missing)?;

        let mut si_name_attribute_prototype = si_name_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;

        // Create and set the func to take off a string field.
        let transformation_func_name = "si:getRegion";
        let transformation_func = Func::find_by_attr(ctx, "name", &transformation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(transformation_func_name.to_owned()))?;

        let transformation_func_argument =
            FuncArgument::find_by_name_for_func(ctx, "region", *transformation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        transformation_func_name.to_owned(),
                        "region".to_string(),
                    )
                })?;

        si_name_attribute_prototype
            .set_func_id(ctx, transformation_func.id())
            .await?;

        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *si_name_attribute_prototype.id(),
            *transformation_func_argument.id(),
            *region_implicit_internal_provider.id(),
        )
        .await?;

        // Connect the "/root/domain/region" prop to the external provider.
        let external_provider_attribute_prototype_id = region_external_provider
            .attribute_prototype_id()
            .ok_or_else(|| {
                BuiltinsError::MissingAttributePrototypeForExternalProvider(
                    *region_external_provider.id(),
                )
            })?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *external_provider_attribute_prototype_id,
            identity_func_item.func_argument_id,
            *region_implicit_internal_provider.id(),
        )
        .await?;

        let region_refresh_workflow_name = "si:awsRegionRefreshWorkflow";
        let region_refresh_workflow_func =
            Func::find_by_attr(ctx, "name", &region_refresh_workflow_name)
                .await?
                .pop()
                .ok_or_else(|| {
                    SchemaError::FuncNotFound(region_refresh_workflow_name.to_owned())
                })?;
        let title = "Refresh Region";
        let context = WorkflowPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        let workflow_prototype = WorkflowPrototype::new(
            ctx,
            *region_refresh_workflow_func.id(),
            serde_json::Value::Null,
            context,
            title,
        )
        .await?;

        let name = "refresh";
        let context = ActionPrototypeContext {
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            ..Default::default()
        };
        ActionPrototype::new(
            ctx,
            *workflow_prototype.id(),
            name,
            ActionKind::Other,
            context,
        )
        .await?;

        Ok(())
    }
}
