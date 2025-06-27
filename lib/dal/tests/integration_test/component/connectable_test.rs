use dal::{
    Component,
    ComponentId,
    ComponentType,
    DalContext,
    FuncId,
    InputSocket,
    OutputSocket,
    OutputSocketId,
    SchemaVariant,
    SchemaVariantId,
};
use dal_test::{
    Result,
    expected::{
        ExpectComponent,
        ExpectFunc,
    },
    helpers::{
        ChangeSetTestHelpers,
        create_component_for_schema_variant_on_default_view,
        get_attribute_value_for_component,
        schema::variant,
        update_attribute_value_for_component,
    },
};
use serde_json::Value;
use si_frontend_types::RawGeometry;

#[derive(Debug, Clone, Copy)]
pub struct ConnectableTest {
    pub connectable_variant_id: SchemaVariantId,
    pub connectable_manager_variant_id: SchemaVariantId,
    pub retrieve_managed_values_id: FuncId,
}

impl ConnectableTest {
    pub async fn setup(ctx: &DalContext) -> Result<Self> {
        let connectable_variant_id = variant::create(
            ctx,
            "connectable",
            r#"
                function main() {
                    return {
                        props: [
                            { name: "Value", kind: "string" },
                            { name: "One", kind: "string", valueFrom: { kind: "inputSocket", socket_name: "One" } },
                            { name: "Many", kind: "array",
                                entry: { name: "ManyItem", kind: "string" },
                                valueFrom: { kind: "inputSocket", socket_name: "Many" },
                            },
                            { name: "Inferred", kind: "string", valueFrom: { kind: "inputSocket", socket_name: "Inferred" } },
                            { name: "Missing", kind: "string", valueFrom: { kind: "inputSocket", socket_name: "Missing" } },
                            { name: "Empty", kind: "array",
                                entry: { name: "EmptyItem", kind: "string" },
                                valueFrom: { kind: "inputSocket", socket_name: "Empty" },
                            },
                        ],
                        inputSockets: [
                            { name: "One", arity: "one", connectionAnnotations: "[\"Value\"]" },
                            { name: "Many", arity: "many", connectionAnnotations: "[\"Value\"]" },
                            { name: "Missing", arity: "one", connectionAnnotations: "[\"Value\"]" },
                            { name: "Empty", arity: "many", connectionAnnotations: "[\"Value\"]" },
                            { name: "Inferred", arity: "one", connectionAnnotations: "[\"Inferred\"]" },
                        ],
                        outputSockets: [
                            { name: "Value", arity: "one", valueFrom: { kind: "prop", prop_path: [ "root", "domain", "Value" ] }, connectionAnnotations: "[\"Value\"]" },
                        ],
                    };
                }
            "#,
        )
        .await?;
        let connectable_manager_variant_id = variant::create(ctx,
            "connectable manager",
            r#"
                function main() {
                    return {
                        props: [
                            { name: "Value", kind: "string" },
                            { name: "ManagedValues", kind: "array",
                                entry: { name: "ManagedValuesItem", kind: "string" },
                            },
                        ],
                        outputSockets: [
                            { name: "Inferred", arity: "one", valueFrom: { kind: "prop", prop_path: [ "root", "domain", "Value" ] }, connectionAnnotations: "[\"Inferred\"]" },
                        ],
                    };
                }
            "#,
        )
        .await?;
        SchemaVariant::get_by_id(ctx, connectable_manager_variant_id)
            .await?
            .set_type(ctx, ComponentType::ConfigurationFrameDown)
            .await?;
        let retrieve_managed_values_id = variant::create_management_func(
                ctx,
                "connectable manager",
                "retrieve_managed_values",
                r#"
                    function main(input) {
                        let managed_values = Object.values(input.components).map(c => c.properties.domain.Value).sort();
                        return {
                            status: "ok",
                            ops: {
                                update: {
                                    self: {
                                        properties: {
                                            domain: {
                                                ManagedValues: managed_values
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                "#,
            )
            .await?;
        Ok(Self {
            connectable_variant_id,
            connectable_manager_variant_id,
            retrieve_managed_values_id,
        })
    }

    pub async fn create_connectable(
        self,
        ctx: &DalContext,
        name: &str,
        connect_one: Option<Connectable>,
        connect_many: impl IntoIterator<Item = Connectable>,
    ) -> Result<Connectable> {
        let one =
            InputSocket::find_with_name_or_error(ctx, "One", self.connectable_variant_id).await?;
        let many =
            InputSocket::find_with_name_or_error(ctx, "Many", self.connectable_variant_id).await?;

        let connectable = {
            let component = create_component_for_schema_variant_on_default_view(
                ctx,
                self.connectable_variant_id,
            )
            .await?;
            component.set_name(ctx, name).await?;
            Component::set_type_by_id(ctx, component.id(), ComponentType::ConfigurationFrameDown)
                .await?;
            Connectable::new(self, component.id())
        };

        connectable.set_value(ctx, name).await?;

        if let Some(from) = connect_one {
            Component::connect(
                ctx,
                from.id,
                from.value_output_socket_id(ctx).await?,
                connectable.id,
                one.id(),
            )
            .await?;
        }
        for from in connect_many {
            Component::connect(
                ctx,
                from.id,
                from.value_output_socket_id(ctx).await?,
                connectable.id,
                many.id(),
            )
            .await?;
        }

        Ok(connectable)
    }

    pub async fn create_parent(self, ctx: &DalContext, name: &str) -> Result<Connectable> {
        let component = create_component_for_schema_variant_on_default_view(
            ctx,
            self.connectable_manager_variant_id,
        )
        .await?;
        component.set_name(ctx, name).await?;
        update_attribute_value_for_component(
            ctx,
            component.id(),
            &["root", "domain", "Value"],
            name.into(),
        )
        .await?;
        Ok(Connectable::new(self, component.id()))
    }

    pub async fn create_manager(self, ctx: &DalContext, name: &str) -> Result<Connectable> {
        self.create_parent(ctx, name).await
    }
}

// Component with output socket "Value" and input sockets "One", "Many", "Inferred", "Missing", and
// "Empty" which can connect to "Value".
#[derive(Debug, Copy, Clone, derive_more::From, derive_more::Into)]
pub struct Connectable {
    pub test: ConnectableTest,
    pub id: ComponentId,
}

impl Connectable {
    pub fn new(test: ConnectableTest, id: ComponentId) -> Self {
        Self { test, id }
    }

    pub async fn run_management_func(self, ctx: &mut DalContext) -> Result<Value> {
        ExpectComponent(self.id)
            .execute_management_func(ctx, ExpectFunc(self.test.retrieve_managed_values_id))
            .await;
        ChangeSetTestHelpers::commit_and_update_snapshot_to_visibility(ctx).await?;
        get_attribute_value_for_component(ctx, self.id, &["root", "domain", "ManagedValues"]).await
    }

    pub async fn set_value(self, ctx: &DalContext, value: &str) -> Result<()> {
        update_attribute_value_for_component(
            ctx,
            self.id,
            &["root", "domain", "Value"],
            value.into(),
        )
        .await
    }

    // Get the domain, with the Many prop sorted
    pub async fn domain(self, ctx: &DalContext) -> Result<Value> {
        let mut domain =
            get_attribute_value_for_component(ctx, self.id, &["root", "domain"]).await?;
        if let Some(many) = domain.get_mut("Many") {
            let many = many.as_array_mut().expect("Many is an array");
            many.sort_by_key(|v| v.as_str().expect("Many is an array of strings").to_string());
        }
        Ok(domain)
    }

    pub async fn value_output_socket_id(self, ctx: &DalContext) -> Result<OutputSocketId> {
        let variant_id = Component::schema_variant_id(ctx, self.id).await?;
        let value_socket = OutputSocket::find_with_name_or_error(ctx, "Value", variant_id).await?;
        Ok(value_socket.id())
    }
}

pub const GEOMETRY1: RawGeometry = RawGeometry {
    x: 1,
    y: 11,
    width: Some(111),
    height: Some(1111),
};

pub const GEOMETRY2: RawGeometry = RawGeometry {
    x: 2,
    y: 22,
    width: Some(222),
    height: Some(2222),
};
