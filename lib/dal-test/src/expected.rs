#![allow(async_fn_in_trait)]
#![allow(missing_docs)]
#![allow(clippy::expect_used)]

use std::ops::Deref;

use crate::helpers::ChangeSetTestHelpers;
use dal::prop::{Prop, PropPath};
use dal::property_editor::values::PropertyEditorValues;
use dal::{
    AttributeValue, AttributeValueId, ChangeSet, ChangeSetId, ComponentId, PropId, SchemaVariantId
};
use dal::{Component, DalContext};
use serde_json::Value;

///
/// Things that you can pass as prop paths
///
pub trait IntoPropPath {
    ///
    /// Convert into a PropPath
    /// 
    fn into_prop_path(self) -> PropPath;
}
impl IntoPropPath for PropPath {
    fn into_prop_path(self) -> PropPath {
        self
    }
}
impl IntoPropPath for &str {
    fn into_prop_path(self) -> PropPath {
        [self].into_prop_path()
    }
}
impl<const N: usize> IntoPropPath for [&str; N] {
    fn into_prop_path(self) -> PropPath {
        PropPath::new(self)
    }
}

#[derive(Debug)]
pub struct ExpectSchemaVariant(pub SchemaVariantId);

impl Deref for ExpectSchemaVariant {
    type Target = SchemaVariantId;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ExpectSchemaVariant {
    pub async fn id(&self) -> SchemaVariantId {
        self.0
    }
    pub async fn prop(&self, ctx: &DalContext, prop_path: impl IntoPropPath) -> ExpectProp {
        ExpectProp(ExpectProp::find_prop_id_by_path(ctx, self.0, prop_path).await)
    }
    pub async fn domain_prop(&self, ctx: &DalContext, prop_path: impl IntoPropPath) -> ExpectProp {
        let prop_path = PropPath::new(["root", "domain"]).join(&prop_path.into_prop_path());
        ExpectProp(ExpectProp::find_prop_id_by_path(ctx, self.0, prop_path).await)
    }
}

#[derive(Debug)]
pub struct ExpectComponent(pub ComponentId);

impl Deref for ExpectComponent {
    type Target = ComponentId;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ExpectComponent {
    pub fn id(&self) -> ComponentId {
        self.0
    }

    pub async fn component(&self, ctx: &DalContext) -> Component {
        Component::get_by_id(ctx, self.0).await.expect("find component by id")
    }

    pub async fn schema_variant(&self, ctx: &DalContext) -> ExpectSchemaVariant {
        let schema_variant_id = Component::schema_variant_id(ctx, self.id())
            .await
            .expect("find variant id for component");
        ExpectSchemaVariant(schema_variant_id)
    }

    pub async fn domain_prop(&self, ctx: &DalContext, path: impl IntoPropPath) -> ExpectComponentProp {
        let prop = self.schema_variant(ctx).await.domain_prop(ctx, path).await;
        ExpectComponentProp(self.id(), prop.id())
    }

    pub async fn prop(&self, ctx: &DalContext, path: impl IntoPropPath) -> ExpectComponentProp {
        let prop = self.schema_variant(ctx).await.prop(ctx, path).await;
        ExpectComponentProp(self.id(), prop.id())
    }

    pub async fn value(&self, ctx: &DalContext) -> Value {
        self.component(ctx).await.view(ctx).await.expect("component must have value").expect("component must have value")
    }
    pub async fn view(&self, ctx: &DalContext) -> Option<Value> {
        self.component(ctx).await.view(ctx).await.expect("component must have value")
    }

    pub async fn attribute_values_for_prop_id(
        ctx: &DalContext,
        component_id: ComponentId,
        prop_id: PropId,
    ) -> Vec<AttributeValueId> {
        Component::attribute_values_for_prop_id(ctx, component_id, prop_id)
            .await
            .expect("able to list prop values")
    }
}

#[derive(Debug)]
pub struct ExpectComponentProp(ComponentId, PropId);

impl ExpectComponentProp {
    pub fn component_id(&self) -> ComponentId {
        self.0
    }
    pub fn prop_id(&self) -> PropId {
        self.1
    }
    pub async fn attribute_value_id(&self, ctx: &DalContext) -> AttributeValueId {
        let prop_values = ExpectPropertyEditorValues::assemble(ctx, self.0).await;
        prop_values
            .find_by_prop_id(self.1)
            .expect("able to find prop value")
    }
    pub async fn attribute_value(&self, ctx: &DalContext) -> ExpectAttributeValue {
        let attribute_value_id = self.attribute_value_id(ctx).await;
        ExpectAttributeValue::get_by_id(ctx, attribute_value_id).await
    }

    pub async fn get(&self, ctx: &DalContext) -> Value {
        self.attribute_value(ctx).await.get(ctx).await
    }
    pub async fn set(&self, ctx: &DalContext, value: impl Into<Value>) {
        self.update(ctx, Some(value.into())).await
    }
    pub async fn push(&self, ctx: &DalContext, value: impl Into<Value>) -> AttributeValueId {
        self.insert(ctx, Some(value.into()), None).await
    }
    pub async fn child_at(&self, ctx: &DalContext, index: usize) -> ExpectAttributeValue {
        self.attribute_value(ctx).await.child_at(ctx, index).await
    }
    pub async fn remove_child_at(&self, ctx: &DalContext, index: usize) {
        self.attribute_value(ctx)
            .await
            .remove_child_at(ctx, index)
            .await
    }

    pub async fn view(&self, ctx: &DalContext) -> Option<Value> {
        self.attribute_value(ctx).await.view(ctx).await
    }
    pub async fn update(&self, ctx: &DalContext, value: Option<Value>) {
        ExpectAttributeValue::update(ctx, self.attribute_value_id(ctx).await, value).await
    }
    pub async fn insert(
        &self,
        ctx: &DalContext,
        value: Option<Value>,
        key: Option<String>,
    ) -> AttributeValueId {
        ExpectAttributeValue::insert(ctx, self.attribute_value_id(ctx).await, value, key).await
    }
}

#[derive(Debug)]
pub struct ExpectAttributeValue(pub AttributeValue);

impl Deref for ExpectAttributeValue {
    type Target = AttributeValue;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ExpectAttributeValue {
    pub async fn view(&self, ctx: &DalContext) -> Option<Value> {
        self.0.view(ctx).await.expect("attribute value view")
    }
    pub async fn get(&self, ctx: &DalContext) -> Value {
        self.view(ctx).await.expect("attribute must have value")
    }
    pub async fn child_at(&self, ctx: &DalContext, index: usize) -> ExpectAttributeValue {
        let child_ids = ExpectAttributeValue::get_child_av_ids_in_order(ctx, self.0.id).await;
        ExpectAttributeValue::get_by_id(ctx, child_ids[index]).await
    }
    pub async fn set(&self, ctx: &DalContext, value: impl Into<Value>) {
        ExpectAttributeValue::update(ctx, self.0.id, Some(value.into())).await
    }
    pub async fn remove_child_at(&self, ctx: &DalContext, index: usize) {
        let child_ids = ExpectAttributeValue::get_child_av_ids_in_order(ctx, self.0.id).await;
        ExpectAttributeValue::remove_by_id(ctx, child_ids[index]).await
    }

    pub async fn update(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<Value>,
    ) {
        AttributeValue::update(ctx, attribute_value_id, value)
            .await
            .expect("update prop value failed")
    }
    pub async fn insert(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
        value: Option<Value>,
        key: Option<String>,
    ) -> AttributeValueId {
        AttributeValue::insert(ctx, attribute_value_id, value, key)
            .await
            .expect("insert prop value failed")
    }
    pub async fn get_by_id(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> ExpectAttributeValue {
        ExpectAttributeValue(
            AttributeValue::get_by_id_or_error(ctx, attribute_value_id)
                .await
                .expect("get prop value by id failed"),
        )
    }
    pub async fn get_child_av_ids_in_order(
        ctx: &DalContext,
        attribute_value_id: AttributeValueId,
    ) -> Vec<AttributeValueId> {
        AttributeValue::get_child_av_ids_in_order(ctx, attribute_value_id)
            .await
            .expect("get child av ids in order")
    }
    pub async fn remove_by_id(ctx: &DalContext, attribute_value_id: AttributeValueId) {
        AttributeValue::remove_by_id(ctx, attribute_value_id)
            .await
            .expect("remove prop value by id failed")
    }
}

#[derive(Debug)]
pub struct ExpectProp(pub PropId);

impl Deref for ExpectProp {
    type Target = PropId;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ExpectProp {
    pub fn id(&self) -> PropId { self.0 }

    pub async fn child(&self, ctx: &DalContext, path: impl IntoPropPath) -> ExpectProp {
        ExpectProp(Self::find_child_prop_id_by_path(ctx, self.0, path).await)
    }

    pub async fn find_child_prop_id_by_path(
        ctx: &DalContext,
        prop_id: impl Into<PropId>,
        path: impl IntoPropPath,
    ) -> PropId {
        Prop::find_child_prop_id_by_path(ctx, prop_id, &path.into_prop_path())
            .await
            .expect("able to find child prop")
    }
    pub async fn find_prop_id_by_path(
        ctx: &DalContext,
        schema_variant_id: impl Into<SchemaVariantId>,
        path: impl IntoPropPath,
    ) -> PropId {
        Prop::find_prop_id_by_path(ctx, schema_variant_id.into(), &path.into_prop_path())
            .await
            .expect("able to find prop")
    }
    pub async fn direct_single_child_prop_id(ctx: &DalContext, prop_id: PropId) -> PropId {
        Prop::direct_single_child_prop_id(ctx, prop_id)
            .await
            .expect("able to find element prop")
    }
}

#[derive(Debug)]
pub struct ExpectPropertyEditorValues;

impl ExpectPropertyEditorValues {
    pub async fn assemble(ctx: &DalContext, id: impl Into<ComponentId>) -> PropertyEditorValues {
        PropertyEditorValues::assemble(ctx, id.into())
            .await
            .expect("able to list prop values")
    }
}

pub async fn apply_change_set_to_base(ctx: &mut DalContext) {
    ChangeSetTestHelpers::apply_change_set_to_base(ctx)
        .await
        .expect("could not apply change set to base");
}

pub async fn fork_from_head_change_set(ctx: &mut DalContext) -> ChangeSet {
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("fork from head")
}

pub async fn create_component(ctx: &mut DalContext, schema_name: impl AsRef<str>) -> ExpectComponent {
    ExpectComponent(
        crate::helpers::create_component(ctx, schema_name)
            .await
            .expect("could not create component")
            .id()
    )
}

pub async fn create_component_for_default_schema_name(
    ctx: &mut DalContext,
    schema_name: impl AsRef<str>,
    name: impl AsRef<str>,
) -> Component {
    crate::helpers::create_component_for_default_schema_name(ctx, schema_name, name)
        .await
        .expect("could not create component")
}

pub async fn update_visibility_and_snapshot_to_visibility(
    ctx: &mut DalContext,
    change_set_id: ChangeSetId,
) {
    ctx.update_visibility_and_snapshot_to_visibility(change_set_id)
        .await
        .expect("could not update visibility and snapshot")
}

pub async fn commit_and_update_snapshot_to_visibility(ctx: &mut DalContext) {
    ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx)
        .await
        .expect("could not commit and update snapshot")
}
