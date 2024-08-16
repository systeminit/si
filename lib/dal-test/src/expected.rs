#![allow(async_fn_in_trait)]
#![allow(missing_docs)]
#![allow(clippy::expect_used)]

use crate::helpers::ChangeSetTestHelpers;
use dal::{
    self,
    prop::{Prop, PropPath},
    property_editor::values::PropertyEditorValues,
    schema::variant::authoring::VariantAuthoringClient,
    AttributeValue, AttributeValueId, ChangeSetId, Component, ComponentId, ComponentType,
    DalContext, PropId, Schema, SchemaId, SchemaVariant, SchemaVariantId,
};
use derive_more::{AsMut, AsRef, Deref, From, Into};
use serde_json::Value;

///
/// Things that you can pass as prop paths / ids
///
pub trait ExpectPropId {
    ///
    /// Turn this into a proper prop id
    ///
    async fn expect_prop_id(self, ctx: &DalContext, schema_variant: ExpectSchemaVariant) -> PropId;
}
impl ExpectPropId for PropId {
    async fn expect_prop_id(self, _: &DalContext, _: ExpectSchemaVariant) -> PropId {
        self
    }
}
impl ExpectPropId for ExpectComponentProp {
    async fn expect_prop_id(self, _: &DalContext, _: ExpectSchemaVariant) -> PropId {
        self.prop().id()
    }
}
impl ExpectPropId for ExpectProp {
    async fn expect_prop_id(self, _: &DalContext, _: ExpectSchemaVariant) -> PropId {
        self.id()
    }
}
impl ExpectPropId for PropPath {
    async fn expect_prop_id(self, ctx: &DalContext, schema_variant: ExpectSchemaVariant) -> PropId {
        schema_variant.find_prop_id_by_path(ctx, &self).await
    }
}
impl<const N: usize> ExpectPropId for [&str; N] {
    async fn expect_prop_id(self, ctx: &DalContext, schema_variant: ExpectSchemaVariant) -> PropId {
        self.expect_prop_path()
            .expect_prop_id(ctx, schema_variant)
            .await
    }
}

///
/// Things that you can pass as prop paths
///
pub trait ExpectPropPath {
    ///
    /// Turn this into a proper prop path
    ///
    fn expect_prop_path(self) -> PropPath;
}
impl ExpectPropPath for PropPath {
    fn expect_prop_path(self) -> PropPath {
        self
    }
}
impl<const N: usize> ExpectPropPath for [&str; N] {
    fn expect_prop_path(self) -> PropPath {
        PropPath::new(self)
    }
}

///
/// Things that you can pass as schema ids
///
pub trait ExpectSchemaId {
    ///
    /// Turn this into a real SchemaId
    ///
    async fn expect_schema_id(&self, ctx: &DalContext) -> SchemaId;
}
impl ExpectSchemaId for SchemaId {
    async fn expect_schema_id(&self, _: &DalContext) -> SchemaId {
        *self
    }
}
impl ExpectSchemaId for ExpectSchema {
    async fn expect_schema_id(&self, _: &DalContext) -> SchemaId {
        self.id()
    }
}
impl ExpectSchemaId for Schema {
    async fn expect_schema_id(&self, _: &DalContext) -> SchemaId {
        self.id()
    }
}
impl ExpectSchemaId for str {
    async fn expect_schema_id(&self, ctx: &DalContext) -> SchemaId {
        Schema::find_by_name(ctx, self)
            .await
            .expect("find schema by name")
            .expect("schema exists")
            .id()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deref, AsRef, From, Into)]
pub struct ExpectSchema(pub SchemaId);

impl From<Schema> for ExpectSchema {
    fn from(schema: Schema) -> Self {
        ExpectSchema(schema.id())
    }
}

impl ExpectSchema {
    pub async fn find(ctx: &DalContext, name: impl AsRef<str>) -> ExpectSchema {
        ExpectSchema(name.as_ref().expect_schema_id(ctx).await)
    }

    pub fn id(self) -> SchemaId {
        self.0
    }

    pub async fn schema(self, ctx: &DalContext) -> Schema {
        Schema::get_by_id(ctx, self.0)
            .await
            .expect("get schema by id")
    }

    pub async fn default_variant(self, ctx: &DalContext) -> ExpectSchemaVariant {
        let schema = self.schema(ctx).await;
        let schema_variant_id = schema
            .get_default_schema_variant_id(ctx)
            .await
            .expect("get default variant id")
            .expect("default variant exists");
        ExpectSchemaVariant(schema_variant_id)
    }

    pub async fn create_component(self, ctx: &DalContext) -> ExpectComponent {
        self.default_variant(ctx).await.create_component(ctx).await
    }

    pub async fn create_named_component(
        self,
        ctx: &DalContext,
        name: impl AsRef<str>,
    ) -> ExpectComponent {
        self.default_variant(ctx)
            .await
            .create_named_component(ctx, name)
            .await
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deref, AsRef, From, Into)]
pub struct ExpectSchemaVariant(pub SchemaVariantId);

impl From<SchemaVariant> for ExpectSchemaVariant {
    fn from(variant: SchemaVariant) -> Self {
        ExpectSchemaVariant(variant.id())
    }
}

impl ExpectSchemaVariant {
    pub fn id(self) -> SchemaVariantId {
        self.0
    }

    pub async fn schema_variant(self, ctx: &DalContext) -> SchemaVariant {
        SchemaVariant::get_by_id_or_error(ctx, self.0)
            .await
            .expect("find schema variant by id")
    }

    pub async fn get_type(self, ctx: &DalContext) -> ComponentType {
        self.schema_variant(ctx)
            .await
            .get_type(ctx)
            .await
            .expect("get type")
            .expect("has type")
    }

    pub async fn set_type(self, ctx: &DalContext, component_type: ComponentType) {
        self.schema_variant(ctx)
            .await
            .set_type(ctx, component_type)
            .await
            .expect("set type")
    }

    pub async fn find_prop_id_by_path(self, ctx: &DalContext, path: &PropPath) -> PropId {
        Prop::find_prop_id_by_path(ctx, self.0, path)
            .await
            .expect("able to find prop")
    }

    pub async fn create_unlocked_copy(self, ctx: &DalContext) -> ExpectSchemaVariant {
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, self.0)
            .await
            .expect("create unlocked variant copy")
            .into()
    }

    pub async fn create_component(self, ctx: &DalContext) -> ExpectComponent {
        self.create_named_component(ctx, generate_fake_name()).await
    }

    pub async fn create_named_component(
        self,
        ctx: &DalContext,
        name: impl AsRef<str>,
    ) -> ExpectComponent {
        ExpectComponent(
            Component::new(ctx, name.as_ref().to_string(), self.0)
                .await
                .expect("create component")
                .id(),
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deref, AsRef, From, Into)]
pub struct ExpectComponent(pub ComponentId);

impl From<Component> for ExpectComponent {
    fn from(component: Component) -> Self {
        ExpectComponent(component.id())
    }
}

impl ExpectComponent {
    pub async fn create(ctx: &mut DalContext, schema_name: impl AsRef<str>) -> ExpectComponent {
        ExpectSchema::find(ctx, schema_name)
            .await
            .create_component(ctx)
            .await
    }

    pub async fn create_named(
        ctx: &mut DalContext,
        schema_name: impl AsRef<str>,
        name: impl AsRef<str>,
    ) -> ExpectComponent {
        ExpectSchema::find(ctx, schema_name)
            .await
            .create_named_component(ctx, name)
            .await
    }

    pub fn id(self) -> ComponentId {
        self.0
    }

    pub async fn component(self, ctx: &DalContext) -> Component {
        Component::get_by_id(ctx, self.0)
            .await
            .expect("get component by id")
    }

    pub async fn get_type(self, ctx: &DalContext) -> ComponentType {
        dal::Component::get_type_by_id(ctx, self.0)
            .await
            .expect("get type by id")
    }

    pub async fn set_type(self, ctx: &DalContext, component_type: ComponentType) {
        self.component(ctx)
            .await
            .set_type(ctx, component_type)
            .await
            .expect("set type")
    }

    pub async fn schema_variant(self, ctx: &DalContext) -> ExpectSchemaVariant {
        ExpectSchemaVariant(
            dal::Component::schema_variant_id(ctx, self.0)
                .await
                .expect("get schema variant id"),
        )
    }

    pub async fn prop(self, ctx: &DalContext, prop: impl ExpectPropId) -> ExpectComponentProp {
        let schema_variant = self.schema_variant(ctx).await;
        let prop_id = prop.expect_prop_id(ctx, schema_variant).await;
        ExpectComponentProp(self.0, prop_id)
    }

    pub async fn upsert_parent(self, ctx: &DalContext, parent_id: impl Into<ComponentId>) {
        dal::component::frame::Frame::upsert_parent(ctx, self.0, parent_id.into())
            .await
            .expect("could not upsert parent")
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ExpectComponentProp(ComponentId, PropId);

impl ExpectComponentProp {
    pub fn component(self) -> ExpectComponent {
        ExpectComponent(self.0)
    }
    pub fn prop(self) -> ExpectProp {
        ExpectProp(self.1)
    }

    pub async fn attribute_value(self, ctx: &DalContext) -> ExpectAttributeValue {
        let prop_values = ExpectPropertyEditorValues::assemble(ctx, self.0).await;
        let attribute_value_id = prop_values
            .find_by_prop_id(self.1)
            .expect("able to find prop value");
        ExpectAttributeValue(attribute_value_id)
    }

    pub async fn get(self, ctx: &DalContext) -> Value {
        self.attribute_value(ctx).await.get(ctx).await
    }

    pub async fn set(self, ctx: &DalContext, value: impl Into<Value>) {
        self.attribute_value(ctx)
            .await
            .update(ctx, Some(value.into()))
            .await
    }

    pub async fn push(self, ctx: &DalContext, value: impl Into<Value>) -> AttributeValueId {
        self.attribute_value(ctx)
            .await
            .insert(ctx, Some(value.into()), None)
            .await
    }

    pub async fn children(self, ctx: &DalContext) -> Vec<ExpectAttributeValue> {
        self.attribute_value(ctx).await.children(ctx).await
    }

    pub async fn view(self, ctx: &DalContext) -> Option<Value> {
        self.attribute_value(ctx).await.view(ctx).await
    }

    pub async fn update(self, ctx: &DalContext, value: Option<Value>) {
        self.attribute_value(ctx).await.update(ctx, value).await
    }

    pub async fn insert(
        self,
        ctx: &DalContext,
        value: Option<Value>,
        key: Option<String>,
    ) -> AttributeValueId {
        self.attribute_value(ctx)
            .await
            .insert(ctx, value, key)
            .await
    }

    pub async fn attribute_values_for_prop(self, ctx: &DalContext) -> Vec<ExpectAttributeValue> {
        Component::attribute_values_for_prop_id(ctx, self.0, self.1)
            .await
            .expect("able to list prop values")
            .into_iter()
            .map(ExpectAttributeValue)
            .collect()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deref, AsRef, AsMut, From, Into)]
pub struct ExpectAttributeValue(pub AttributeValueId);

impl From<AttributeValue> for ExpectAttributeValue {
    fn from(value: AttributeValue) -> Self {
        ExpectAttributeValue(value.id())
    }
}

impl ExpectAttributeValue {
    pub fn id(self) -> AttributeValueId {
        self.0
    }

    pub async fn attribute_value(self, ctx: &DalContext) -> AttributeValue {
        dal::AttributeValue::get_by_id_or_error(ctx, self.0)
            .await
            .expect("get prop value by id failed")
    }

    pub async fn view(self, ctx: &DalContext) -> Option<Value> {
        self.attribute_value(ctx)
            .await
            .view(ctx)
            .await
            .expect("attribute value view")
    }

    pub async fn get(self, ctx: &DalContext) -> Value {
        self.view(ctx).await.expect("attribute must have value")
    }

    pub async fn children(self, ctx: &DalContext) -> Vec<ExpectAttributeValue> {
        let child_ids = dal::AttributeValue::get_child_av_ids_in_order(ctx, self.0)
            .await
            .expect("get child av ids in order");
        child_ids.into_iter().map(ExpectAttributeValue).collect()
    }

    pub async fn update(self, ctx: &DalContext, value: Option<Value>) {
        dal::AttributeValue::update(ctx, self.0, value)
            .await
            .expect("update prop value failed")
    }

    pub async fn insert(
        self,
        ctx: &DalContext,
        value: Option<Value>,
        key: Option<String>,
    ) -> AttributeValueId {
        dal::AttributeValue::insert(ctx, self.0, value, key)
            .await
            .expect("insert prop value failed")
    }

    pub async fn remove(self, ctx: &DalContext) {
        dal::AttributeValue::remove_by_id(ctx, self.0)
            .await
            .expect("remove prop value by id failed")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deref, AsRef, AsMut, From, Into)]
pub struct ExpectProp(pub PropId);

impl From<Prop> for ExpectProp {
    fn from(prop: Prop) -> Self {
        ExpectProp(prop.id())
    }
}

impl ExpectProp {
    pub fn id(self) -> PropId {
        self.0
    }

    pub async fn prop(self, ctx: &DalContext) -> Prop {
        Prop::get_by_id_or_error(ctx, self.0)
            .await
            .expect("get prop by id")
    }

    pub async fn direct_single_child(self, ctx: &DalContext) -> ExpectProp {
        ExpectProp(
            Prop::direct_single_child_prop_id(ctx, self.0)
                .await
                .expect("able to find element prop"),
        )
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

pub async fn fork_from_head_change_set(ctx: &mut DalContext) -> dal::ChangeSet {
    ChangeSetTestHelpers::fork_from_head_change_set(ctx)
        .await
        .expect("fork from head")
}

pub fn generate_fake_name() -> String {
    crate::helpers::generate_fake_name().expect("could not generate fake name")
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
