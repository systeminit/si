#![allow(async_fn_in_trait)]
#![allow(missing_docs)]
#![allow(clippy::expect_used)]

use dal::{
    self,
    AttributeValue,
    AttributeValueId,
    ChangeSetId,
    Component,
    ComponentId,
    ComponentType,
    DalContext,
    FuncId,
    InputSocket,
    InputSocketId,
    OutputSocket,
    OutputSocketId,
    PropId,
    Schema,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    component::socket::ComponentInputSocket,
    diagram::{
        geometry::RawGeometry,
        view::View,
    },
    func::authoring::FuncAuthoringClient,
    prop::{
        Prop,
        PropPath,
    },
    property_editor::values::PropertyEditorValues,
    schema::variant::authoring::VariantAuthoringClient,
};
use derive_more::{
    AsMut,
    AsRef,
    Deref,
    From,
    Into,
};
use serde_json::Value;

use crate::helpers::{
    ChangeSetTestHelpers,
    component,
    schema::{
        self,
        variant::{
            self,
            SchemaVariantKey,
        },
    },
};

///
/// Things that you can pass as prop paths / ids
///
pub trait PropKey {
    ///
    /// Turn this into a proper prop id
    ///
    async fn lookup_prop(self, ctx: &DalContext, schema_variant_id: SchemaVariantId) -> PropId;
}
impl PropKey for PropId {
    async fn lookup_prop(self, _: &DalContext, _: SchemaVariantId) -> PropId {
        self
    }
}
impl PropKey for ExpectComponentProp {
    async fn lookup_prop(self, _: &DalContext, _: SchemaVariantId) -> PropId {
        self.prop().id()
    }
}
impl PropKey for ExpectProp {
    async fn lookup_prop(self, _: &DalContext, _: SchemaVariantId) -> PropId {
        self.id()
    }
}
impl PropKey for PropPath {
    async fn lookup_prop(self, ctx: &DalContext, schema_variant_id: SchemaVariantId) -> PropId {
        ExpectSchemaVariant(schema_variant_id)
            .prop(ctx, self)
            .await
            .id()
    }
}
impl<const N: usize> PropKey for [&str; N] {
    async fn lookup_prop(self, ctx: &DalContext, schema_variant_id: SchemaVariantId) -> PropId {
        self.into_prop_path()
            .lookup_prop(ctx, schema_variant_id)
            .await
    }
}

///
/// Things that you can pass as prop paths
///
pub trait IntoPropPath {
    ///
    /// Turn this into a proper prop path
    ///
    fn into_prop_path(self) -> PropPath;
}
impl IntoPropPath for PropPath {
    fn into_prop_path(self) -> PropPath {
        self
    }
}
impl<const N: usize> IntoPropPath for [&str; N] {
    fn into_prop_path(self) -> PropPath {
        PropPath::new(self)
    }
}

///
/// Things that you can pass as input_socket ids
///
pub trait InputSocketKey {
    ///
    /// Turn this into a real InputSocketId
    ///
    async fn lookup_input_socket(
        self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketId;
}
impl InputSocketKey for InputSocketId {
    async fn lookup_input_socket(self, _: &DalContext, _: SchemaVariantId) -> InputSocketId {
        self
    }
}
impl InputSocketKey for ExpectInputSocket {
    async fn lookup_input_socket(self, _: &DalContext, _: SchemaVariantId) -> InputSocketId {
        self.id()
    }
}
impl InputSocketKey for InputSocket {
    async fn lookup_input_socket(self, _: &DalContext, _: SchemaVariantId) -> InputSocketId {
        self.id()
    }
}
impl InputSocketKey for &str {
    async fn lookup_input_socket(
        self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> InputSocketId {
        ExpectSchemaVariant(schema_variant_id)
            .input_socket(ctx, self)
            .await
            .id()
    }
}

///
/// Things that you can pass as output_socket ids
///
pub trait OutputSocketKey {
    ///
    /// Turn this into a real OutputSocketId
    ///
    async fn lookup_output_socket(
        self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> OutputSocketId;
}
impl OutputSocketKey for OutputSocketId {
    async fn lookup_output_socket(self, _: &DalContext, _: SchemaVariantId) -> OutputSocketId {
        self
    }
}
impl OutputSocketKey for ExpectOutputSocket {
    async fn lookup_output_socket(self, _: &DalContext, _: SchemaVariantId) -> OutputSocketId {
        self.id()
    }
}
impl OutputSocketKey for OutputSocket {
    async fn lookup_output_socket(self, _: &DalContext, _: SchemaVariantId) -> OutputSocketId {
        self.id()
    }
}
impl OutputSocketKey for &str {
    async fn lookup_output_socket(
        self,
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> OutputSocketId {
        ExpectSchemaVariant(schema_variant_id)
            .output_socket(ctx, self)
            .await
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
        ExpectSchema(schema::id(ctx, name.as_ref()).await.expect("lookup schema"))
    }

    pub async fn create(ctx: &DalContext) -> ExpectSchema {
        Self::create_named(ctx, generate_fake_name()).await
    }

    pub async fn create_named(ctx: &DalContext, name: impl AsRef<str>) -> ExpectSchema {
        Schema::new(ctx, name.as_ref())
            .await
            .expect("create schema")
            .into()
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
        let schema_variant_id = Schema::default_variant_id(ctx, self.0)
            .await
            .expect("get default variant id");
        ExpectSchemaVariant(schema_variant_id)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deref, AsRef, From, Into)]
pub struct ExpectFunc(pub FuncId);

impl From<dal::Func> for ExpectFunc {
    fn from(func: dal::Func) -> Self {
        ExpectFunc(func.id)
    }
}

impl ExpectFunc {
    pub fn id(self) -> FuncId {
        self.0
    }

    pub async fn update_code(self, ctx: &DalContext, code: impl Into<String>) {
        FuncAuthoringClient::save_code(ctx, self.id(), code.into())
            .await
            .expect("update code")
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

    pub async fn create(ctx: &DalContext, asset_func: impl Into<String>) -> ExpectSchemaVariant {
        Self::create_named(ctx, generate_fake_name(), asset_func).await
    }

    pub async fn create_named(
        ctx: &DalContext,
        name: impl Into<String>,
        asset_func: impl Into<String>,
    ) -> ExpectSchemaVariant {
        let variant_id = variant::create(ctx, name, asset_func)
            .await
            .expect("create variant");
        variant_id.into()
    }

    pub async fn regenerate(self, ctx: &DalContext) -> ExpectSchemaVariant {
        VariantAuthoringClient::regenerate_variant(ctx, self.0)
            .await
            .expect("regenerate variant")
            .into()
    }

    pub async fn schema(self, ctx: &DalContext) -> ExpectSchema {
        SchemaVariant::schema_id(ctx, self.0)
            .await
            .expect("get schema by id")
            .into()
    }

    pub async fn schema_variant(self, ctx: &DalContext) -> SchemaVariant {
        SchemaVariant::get_by_id(ctx, self.0)
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

    pub async fn prop(self, ctx: &DalContext, path: impl IntoPropPath) -> ExpectProp {
        Prop::find_prop_id_by_path(ctx, self.0, &path.into_prop_path())
            .await
            .expect("able to find prop")
            .into()
    }

    pub async fn input_socket(self, ctx: &DalContext, name: impl AsRef<str>) -> ExpectInputSocket {
        InputSocket::find_with_name(ctx, name, self.0)
            .await
            .expect("could not perform find with name")
            .expect("input socket not found")
            .into()
    }

    pub async fn output_socket(
        self,
        ctx: &DalContext,
        name: impl AsRef<str>,
    ) -> ExpectOutputSocket {
        OutputSocket::find_with_name(ctx, name, self.0)
            .await
            .expect("could not perform find with name")
            .expect("output socket not found")
            .into()
    }

    pub async fn create_unlocked_copy(self, ctx: &DalContext) -> ExpectSchemaVariant {
        VariantAuthoringClient::create_unlocked_variant_copy(ctx, self.0)
            .await
            .expect("create unlocked variant copy")
            .into()
    }

    pub async fn create_component_on_default_view(self, ctx: &DalContext) -> ExpectComponent {
        ExpectComponent(
            component::create(ctx, self, generate_fake_name())
                .await
                .expect("create component"),
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
    pub async fn create(ctx: &mut DalContext, variant: impl SchemaVariantKey) -> ExpectComponent {
        Self::create_named(ctx, variant, generate_fake_name()).await
    }

    pub async fn create_named(
        ctx: &mut DalContext,
        variant: impl SchemaVariantKey,
        name: impl AsRef<str>,
    ) -> ExpectComponent {
        ExpectComponent(
            component::create(ctx, variant, name)
                .await
                .expect("create component"),
        )
    }

    pub async fn list(ctx: &DalContext) -> Vec<ExpectComponent> {
        Component::list_ids(ctx)
            .await
            .expect("list components")
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub async fn find_opt(ctx: &DalContext, name: impl AsRef<str>) -> Option<ExpectComponent> {
        for component in ExpectComponent::list(ctx).await {
            if name.as_ref() == component.name(ctx).await {
                return Some(component);
            }
        }
        None
    }

    pub async fn find(ctx: &DalContext, name: impl AsRef<str>) -> ExpectComponent {
        Self::find_opt(ctx, name)
            .await
            .expect("component not found")
    }

    pub fn id(self) -> ComponentId {
        self.0
    }

    pub async fn component(self, ctx: &DalContext) -> Component {
        Component::get_by_id(ctx, self.0)
            .await
            .expect("get component by id")
    }

    pub async fn name(&self, ctx: &DalContext) -> String {
        Component::name_by_id(ctx, self.0)
            .await
            .expect("get component name")
    }

    pub async fn geometry_for_default(self, ctx: &DalContext) -> RawGeometry {
        let view_id = View::get_id_for_default(ctx)
            .await
            .expect("get default view");

        self.component(ctx)
            .await
            .geometry(ctx, view_id)
            .await
            .expect("get geometry for component")
            .into_raw()
    }

    pub async fn view(self, ctx: &DalContext) -> Option<serde_json::Value> {
        self.component(ctx)
            .await
            .view(ctx)
            .await
            .expect("get component value")
    }

    pub async fn get_type(self, ctx: &DalContext) -> ComponentType {
        Component::get_type_by_id(ctx, self.0)
            .await
            .expect("get type by id")
    }

    pub async fn set_type(self, ctx: &DalContext, component_type: ComponentType) {
        Component::set_type_by_id(ctx, self.id(), component_type)
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

    pub async fn prop(self, ctx: &DalContext, prop: impl PropKey) -> ExpectComponentProp {
        let schema_variant = self.schema_variant(ctx).await;
        let prop_id = prop.lookup_prop(ctx, schema_variant.id()).await;
        ExpectComponentProp(self.0, prop_id)
    }

    pub async fn domain(self, ctx: &DalContext) -> Value {
        self.prop(ctx, ["root", "domain"]).await.get(ctx).await
    }

    pub async fn connect(
        self,
        ctx: &DalContext,
        socket: impl OutputSocketKey,
        dest_component: ExpectComponent,
        dest_socket: impl InputSocketKey,
    ) {
        let src = self.output_socket(ctx, socket).await;
        let dest = dest_component.input_socket(ctx, dest_socket).await;
        src.connect(ctx, dest).await
    }

    pub async fn input_socket(
        self,
        ctx: &DalContext,
        input_socket_id: impl InputSocketKey,
    ) -> ExpectComponentInputSocket {
        let schema_variant = self.schema_variant(ctx).await;
        let input_socket_id = input_socket_id
            .lookup_input_socket(ctx, schema_variant.id())
            .await;
        ExpectComponentInputSocket(self.0, input_socket_id)
    }

    pub async fn input_connections(
        self,
        ctx: &DalContext,
        input_socket_id: impl InputSocketKey,
    ) -> Vec<ExpectComponentOutputSocket> {
        self.input_socket(ctx, input_socket_id)
            .await
            .connections(ctx)
            .await
    }

    pub async fn output_socket(
        self,
        ctx: &DalContext,
        output_socket_id: impl OutputSocketKey,
    ) -> ExpectComponentOutputSocket {
        let schema_variant = self.schema_variant(ctx).await;
        let output_socket_id = output_socket_id
            .lookup_output_socket(ctx, schema_variant.id())
            .await;
        ExpectComponentOutputSocket(self.0, output_socket_id)
    }

    pub async fn upsert_parent(self, ctx: &DalContext, parent_id: impl Into<ComponentId>) {
        dal::component::frame::Frame::upsert_parent(ctx, self.0, parent_id.into())
            .await
            .expect("could not upsert parent");
    }

    pub async fn execute_management_func(self, ctx: &DalContext, func: ExpectFunc) {
        component::execute_management_func(ctx, self.0, func.id())
            .await
            .expect("execute management func")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ExpectComponentProp(ComponentId, PropId);

impl ExpectComponentProp {
    pub fn component(self) -> ExpectComponent {
        ExpectComponent(self.0)
    }
    pub fn prop(self) -> ExpectProp {
        ExpectProp(self.1)
    }

    pub async fn name(self, ctx: &DalContext) -> String {
        self.prop().name(ctx).await
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

    pub async fn push_with_key(
        self,
        ctx: &DalContext,
        key: impl Into<String>,
        value: impl Into<Value>,
    ) -> AttributeValueId {
        self.attribute_value(ctx)
            .await
            .insert(ctx, Some(value.into()), Some(key.into()))
            .await
    }

    pub async fn children(self, ctx: &DalContext) -> Vec<ExpectAttributeValue> {
        self.attribute_value(ctx).await.children(ctx).await
    }

    // The value of the prop, or its default value.
    pub async fn view(self, ctx: &DalContext) -> Option<Value> {
        self.attribute_value(ctx).await.view(ctx).await
    }

    // Whether this attribute has a value explicitly set
    pub async fn has_value(self, ctx: &DalContext) -> bool {
        self.attribute_value(ctx).await.has_value(ctx).await
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ExpectComponentInputSocket(ComponentId, InputSocketId);

impl ExpectComponentInputSocket {
    pub fn component(self) -> ExpectComponent {
        ExpectComponent(self.0)
    }
    pub fn input_socket(self) -> ExpectInputSocket {
        ExpectInputSocket(self.1)
    }

    pub async fn connect(self, ctx: &DalContext, dest: ExpectComponentOutputSocket) {
        dest.connect(ctx, self).await
    }

    pub async fn connections(self, ctx: &DalContext) -> Vec<ExpectComponentOutputSocket> {
        let component_input_socket = ComponentInputSocket {
            component_id: self.0,
            input_socket_id: self.1,
            attribute_value_id: self.attribute_value(ctx).await.id(),
        };
        component_input_socket
            .connections(ctx)
            .await
            .expect("get connections")
            .into_iter()
            .map(|(component_id, output_socket_id, _)| {
                ExpectComponentOutputSocket(component_id, output_socket_id)
            })
            .collect()
    }

    pub async fn attribute_value(self, ctx: &DalContext) -> ExpectAttributeValue {
        InputSocket::component_attribute_value_id(ctx, self.1, self.0)
            .await
            .expect("get attribute value for input socket")
            .into()
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
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ExpectComponentOutputSocket(ComponentId, OutputSocketId);

impl ExpectComponentOutputSocket {
    pub fn component(self) -> ExpectComponent {
        ExpectComponent(self.0)
    }
    pub fn output_socket(self) -> ExpectOutputSocket {
        ExpectOutputSocket(self.1)
    }

    pub async fn connect(self, ctx: &DalContext, dest: ExpectComponentInputSocket) {
        Component::connect(ctx, self.0, self.1, dest.0, dest.1)
            .await
            .expect("could not connect components");
    }

    pub async fn attribute_value(self, ctx: &DalContext) -> ExpectAttributeValue {
        OutputSocket::component_attribute_value_id(ctx, self.1, self.0)
            .await
            .expect("get attribute value for output socket")
            .into()
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
        dal::AttributeValue::get_by_id(ctx, self.0)
            .await
            .expect("get prop value by id failed")
    }

    pub async fn view(self, ctx: &DalContext) -> Option<Value> {
        dal::AttributeValue::view(ctx, self.attribute_value(ctx).await.id())
            .await
            .expect("attribute value view")
    }

    // Whether this attribute has a value explicitly set
    pub async fn has_value(self, ctx: &DalContext) -> bool {
        self.attribute_value(ctx)
            .await
            .value(ctx)
            .await
            .expect("get attribute value")
            .is_some()
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
        dal::AttributeValue::remove(ctx, self.0)
            .await
            .expect("remove prop value by id failed")
    }

    pub async fn prop(self, ctx: &DalContext) -> ExpectComponentProp {
        let component_id = AttributeValue::component_id(ctx, self.0)
            .await
            .expect("get component id");
        let prop_id = AttributeValue::prop_id(ctx, self.0)
            .await
            .expect("get prop id");
        ExpectComponentProp(component_id, prop_id)
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
        Prop::get_by_id(ctx, self.0).await.expect("get prop by id")
    }

    pub async fn name(self, ctx: &DalContext) -> String {
        self.prop(ctx).await.name
    }

    pub async fn children(self, ctx: &DalContext) -> Vec<ExpectProp> {
        Prop::direct_child_prop_ids_ordered(ctx, self.0)
            .await
            .expect("get direct child prop ids ordered")
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub async fn child_names(self, ctx: &DalContext) -> Vec<String> {
        let mut result = vec![];
        for child in self.children(ctx).await {
            result.push(child.name(ctx).await);
        }
        result
    }

    pub async fn direct_single_child(self, ctx: &DalContext) -> ExpectProp {
        ExpectProp(
            Prop::direct_single_child_prop_id(ctx, self.0)
                .await
                .expect("able to find element prop"),
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deref, AsRef, AsMut, From, Into)]
pub struct ExpectOutputSocket(pub OutputSocketId);

impl From<OutputSocket> for ExpectOutputSocket {
    fn from(from: OutputSocket) -> Self {
        from.id().into()
    }
}

impl ExpectOutputSocket {
    pub fn id(self) -> OutputSocketId {
        self.0
    }

    pub async fn output_socket(self, ctx: &DalContext) -> OutputSocket {
        OutputSocket::get_by_id(ctx, self.0)
            .await
            .expect("get output socket by id")
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deref, AsRef, AsMut, From, Into)]
pub struct ExpectInputSocket(pub InputSocketId);

impl From<InputSocket> for ExpectInputSocket {
    fn from(from: InputSocket) -> Self {
        from.id().into()
    }
}

impl ExpectInputSocket {
    pub fn id(self) -> InputSocketId {
        self.0
    }

    pub async fn output_socket(self, ctx: &DalContext) -> InputSocket {
        InputSocket::get_by_id(ctx, self.0)
            .await
            .expect("get output socket by id")
    }
}

#[derive(Debug)]
pub struct ExpectPropertyEditorValues;

impl ExpectPropertyEditorValues {
    pub async fn assemble(ctx: &DalContext, id: impl Into<ComponentId>) -> PropertyEditorValues {
        PropertyEditorValues::assemble(ctx, id.into())
            .await
            .expect("unable to list prop values")
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

#[derive(Debug)]
pub struct ExpectView;

impl ExpectView {
    pub async fn create(ctx: &DalContext) -> View {
        let name = generate_fake_name();
        View::new(ctx, name).await.expect("create view")
    }

    pub async fn create_with_name(ctx: &DalContext, name: &str) -> View {
        View::new(ctx, name).await.expect("create view")
    }
}
