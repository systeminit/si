use dal::{
    BuiltinsResult,
    ComponentType,
    DalContext,
    PropKind,
    SchemaId,
    action::prototype::ActionKind,
    pkg::{
        ImportOptions,
        import_pkg_from_pkg,
    },
};
use si_pkg::{
    ActionFuncSpec,
    ManagementFuncSpec,
    PkgSpec,
    PropSpec,
    SchemaSpec,
    SchemaSpecData,
    SchemaVariantSpec,
    SchemaVariantSpecData,
    SiPkg,
    SocketSpec,
    SocketSpecArity,
    SocketSpecData,
    SocketSpecKind,
};

use crate::test_exclusive_schemas::{
    PKG_CREATED_BY,
    PKG_VERSION,
    build_action_func,
    build_asset_func,
    build_management_func,
    create_identity_func,
    legos::bricks::LegoBricks,
};

/// The "small odd lego" has a special importance for our tests. It is a
/// repository of example management functions used for management function
/// integration tests
pub(crate) async fn migrate_test_exclusive_schema_small_odd_lego(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut small_lego_builder = PkgSpec::builder();

    let schema_name = "small odd lego";

    small_lego_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Build Create Action Func
    let create_action_code = "async function main() {
                return { payload: { \"poop\": true }, status: \"ok\" };
            }";

    let fn_name = "test:createActionSmallLego";
    let create_action_func = build_action_func(create_action_code, fn_name)?;

    // Build Refresh Action Func
    let refresh_action_code = "async function main(component: Input): Promise<Output> {
        return { payload: JSON.parse(component.properties.resource?.payload) || { \"poop\": true } , status: \"ok\" };
    }";

    let fn_name = "test:refreshActionSmallLego";
    let refresh_action_func = build_action_func(refresh_action_code, fn_name)?;

    // Build Update Action Func
    let update_action_code = "async function main(component: Input): Promise<Output> {
        return { payload: { \"poonami\": true }, status: \"ok\" };
    }";
    let fn_name = "test:updateActionSmallLego";
    let update_action_func = build_action_func(update_action_code, fn_name)?;

    // Build Delete Action Func
    let delete_action_code = "async function main() {
        return { payload: null, status: \"ok\" };
    }";

    let import_management_func_code =
        "async function main({ thisComponent }: Input): Promise<Output> {
        const thisProperties = thisComponent.properties;
        return {
            status: 'ok',
            ops: {
                update: {
                    self: {
                        properties: {
                            domain: {
                                ...thisProperties.domain,
                                two: 'step',
                            },
                            ...thisProperties
                        }
                    }
                }
            },
            message: 'hello'
        }
    }";
    let import_management_func_name = "test:importManagementSmallLego";
    let import_management_func =
        build_management_func(import_management_func_code, import_management_func_name)?;

    let simple_create_mgmt_func_code = r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        const thisName = thisComponent.properties?.si?.name ?? "unknown";
        let create = {
            [`${thisName}_clone`]: {
                properties: {
                    ...thisComponent.properties,
                },
                geometry: {
                    x: 10,
                    y: 20,
                }
            }
        };

        for (let [id, component] of Object.entries(components)) {
            const name = component.properties?.si?.name ?? "unknown";
            let clone_name = `${name}_clone`;
            if (clone_name in create) {
                clone_name = `${clone_name}-${id}`;
            }
            create[clone_name] = {
                ...component,
            };
        }

        return {
            status: "ok",
            ops: { create }
        }
    }
    "#;

    let clone_me_mgmt_func_name = "test:cloneMeSmallLego";
    let clone_me_mgmt_func =
        build_management_func(simple_create_mgmt_func_code, clone_me_mgmt_func_name)?;

    let update_managed_func_code = r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        const thisName = thisComponent.properties?.si?.name ?? "unknown";

        const update: { [key: string]: unknown } = {};

        for (let [id, component] of Object.entries(components)) {
            let name = component.properties?.si?.name ?? "unknown";
            update[id] = {
                properties: {
                    ...component.properties,
                    si: {
                        ...component.properties?.si,
                        name: `${name} managed by ${thisName}`,
                    }
                },
            };
        }

        return {
            status: "ok",
            ops: { update }
        }
    }
    "#;
    let update_mgmt_func_name = "test:updateManagedComponent";
    let update_mgmt_func = build_management_func(update_managed_func_code, update_mgmt_func_name)?;

    let update_managed_func_in_view_code = r#"
    async function main({ thisComponent, components, currentView }: Input): Promise<Output> {
        const thisName = thisComponent.properties?.si?.name ?? "unknown";

        const update: { [key: string]: unknown } = {};

        for (let [id, component] of Object.entries(components)) {
            let name = component.properties?.si?.name ?? "unknown";
            update[id] = {
                properties: {
                    ...component.properties,
                    si: {
                        ...component.properties?.si,
                        name: `${name} managed by ${thisName}`,
                    }
                },
                geometry: {
                    [currentView]: {
                        x: 1000,
                        y: 750,
                    }
                }
            };
        }

        return {
            status: "ok",
            message: currentView,
            ops: { update },
        }
    }
    "#;
    let update_in_view_mgmt_func_name = "test:updateManagedComponentInView";
    let update_in_view_mgmt_func = build_management_func(
        update_managed_func_in_view_code,
        update_in_view_mgmt_func_name,
    )?;

    let create_and_connect_from_self_func_code = r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        const thisName = thisComponent.properties?.si?.name ?? "unknown";

        let count = parseInt(thisComponent.properties?.si?.resourceId);
        if (isNaN(count) || count < 1) {
            count = 1;
        }

        const create: { [key: string]: unknown } = {};
        const names = [];
        for (let i = 0; i < count; i++) {
            let name = `clone_${i}`;
            names.push(name);
            create[name] = {
                kind: "small even lego",
                properties: { si: { name } },
            };
        }

        return {
            status: "ok",
            ops: {
                create,
                update: {
                    self: {
                        connect: {
                            add: names.map(name => ({ from: "two", to: { component: name, socket: "two" }}))
                        }
                    }
                }
            },
            message: `created ${names.join(", ")}`,
        }
    }
    "#;
    let create_and_connect_from_self_name = "test:createAndConnectFromSelf";
    let create_and_connect_from_self_func = build_management_func(
        create_and_connect_from_self_func_code,
        create_and_connect_from_self_name,
    )?;

    let create_and_connect_to_self_func_code = r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        const thisName = thisComponent.properties?.si?.name ?? "unknown";

        let count = parseInt(thisComponent.properties?.si?.resourceId);
        if (isNaN(count) || count < 1) {
            count = 1;
        }

        let create: { [key: string]: unknown } = {};
        for (let i = 0; i < count; i++) {
            let name = `clone_${i}`;
            create[name] = {
                kind: "small even lego",
                properties: { si: { name } },
                geometry: { x: 10, y: 10 },
                connect: [{
                    from: "one",
                    to: {
                        component: "self",
                        socket: "one",
                    }
                }]
            };
        }

        return {
            status: "ok",
            ops: {
                create,
            }
        }
    }
    "#;
    let create_and_connect_to_self_name = "test:createAndConnectToSelf";
    let create_and_connect_to_self_func = build_management_func(
        create_and_connect_to_self_func_code,
        create_and_connect_to_self_name,
    )?;

    // This will create a small even lego component and connect its inputs to everything
    // connected to the management component's "one" or "arity_one" sockets.
    let create_and_connect_to_inputs_func = build_management_func(
        r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        const allConnections = []
        allConnections.push(...thisComponent.incomingConnections.one);
        if (thisComponent.incomingConnections.arity_one) {
            allConnections.push(thisComponent.incomingConnections.arity_one);
        }

        return {
            status: "ok",
            ops: {
                create: {
                    lego: {
                        kind: "small even lego",
                        connect: allConnections.map((from) => ({ from, to: "two" })),
                    },
                }
            }
        }
    }
        "#,
        "test:createAndConnectToInputs",
    )?;

    // This will connect the "lego" component's inputs to everything connected to the
    // management component's "one" or "arity_one" sockets.
    let connect_to_inputs_func = build_management_func(
        r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        const allConnections = []
        allConnections.push(...thisComponent.incomingConnections.one)
        if (thisComponent.incomingConnections.arity_one) {
            allConnections.push(thisComponent.incomingConnections.arity_one)
        }

        return {
            status: "ok",
            ops: {
                update: {
                    lego: {
                        connect: {
                            add: allConnections.map((from) => ({ from, to: "two" }))
                        }
                    }
                }
            }
        }
    }
        "#,
        "test:connectToInputs",
    )?;

    // This will disconnect the "lego" component's inputs from everything connected to the
    // management component's "one" or "arity_one" sockets.
    let disconnect_from_inputs_func = build_management_func(
        r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        const allConnections = []
        allConnections.push(...thisComponent.incomingConnections.one);
        if (thisComponent.incomingConnections.arity_one) {
            allConnections.push(thisComponent.incomingConnections.arity_one);
        }

        return {
            status: "ok",
            ops: {
                update: {
                    lego: {
                        connect: {
                            remove: allConnections.map((from) => ({ from, to: "two" }))
                        }
                    }
                }
            }
        }
    }
        "#,
        "test:disconnectFromInputs",
    )?;

    // This will grab the manager component's input socket values and put them in test_result.
    let get_input_values_func = build_management_func(
        r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        return {
            status: "ok",
            ops: {
                update: {
                    self: {
                        properties: {
                            domain: {
                                test_result: {
                                    arity_one: thisComponent.incomingConnections.arity_one?.value,
                                    one: thisComponent.incomingConnections.one.map((c) => c.value).sort(),
                                }
                            }
                        }
                    }
                }
            }
        };
    }
        "#,
        "test:getInputValues",
    )?;

    let create_and_connect_to_self_as_children_code = r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        const thisName = thisComponent.properties?.si?.name ?? "unknown";

        let count = parseInt(thisComponent.properties?.si?.resourceId);
        if (isNaN(count) || count < 1) {
            count = 1;
        }

        let create: { [key: string]: unknown } = {};
        for (let i = 0; i < count; i++) {
            let name = `clone_${i}`;
            create[name] = {
                kind: "small even lego",
                properties: { si: { name } },
                parent: "self"
            };
        }

        return {
            status: "ok",
            ops: {
                update: { self: { properties: { si: { type: "configurationFrameDown" } } } },
                create,
            }
        }
    }
    "#;
    let create_and_connect_to_self_as_children_name = "test:createAndConnectToSelfAsChildren";
    let create_and_connect_to_self_as_children_func = build_management_func(
        create_and_connect_to_self_as_children_code,
        create_and_connect_to_self_as_children_name,
    )?;

    let deeply_nested_children_code = r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        const thisName = thisComponent.properties?.si?.name ?? "unknown";

        const count = 10;

        let create: { [key: string]: unknown } = {};
        let prevName = "self";
        for (let i = 0; i < count; i++) {
            let name = `clone_${i}`;
            create[name] = {
                kind: "small odd lego",
                properties: { si: { name, type: "configurationFrameDown" }, },
                parent: prevName,
            };
            prevName =  name;
        }

        return {
            status: "ok",
            ops: {
                update: { self: { properties: { si: { type: "configurationFrameDown" } } } },
                create,
            }
        }
    }
    "#;
    let deeply_nested_children =
        build_management_func(deeply_nested_children_code, "test:deeplyNestedChildren")?;

    let create_component_in_other_views_code = r#"
    async function main({ thisComponent, currentView }: Input): Promise<Output> {
        const thisView = thisComponent.properties?.si?.resourceId ?? currentView;

        const name = `component in ${thisView}`;

        return {
            status: "ok",
            ops: {
                create: {
                    [name]: {
                        geometry: {
                            [currentView]: { x: 100, y: 100 },
                            [thisView]: { x: 15, y: 15 }
                        }
                    }
                }
            }
        }
    }
    "#;

    let create_component_in_other_views = build_management_func(
        create_component_in_other_views_code,
        "test:createComponentsInOtherViews",
    )?;

    let create_view_and_component_in_view_code = r#"
    async function main({ thisComponent, currentView }: Input): Promise<Output> {
        const thisView = thisComponent.properties?.si?.resourceId ?? currentView;

        const componentName = `component in ${thisView}`;

        return {
            status: "ok",
            ops: {
                views: {
                    create: [thisView],
                },
                create: {
                    [componentName]: {
                        geometry: {
                            [thisView]: { x: 315, y: 315 }
                        }
                    }
                }
            }
        }
    }
    "#;
    let create_view_and_component_in_view = build_management_func(
        create_view_and_component_in_view_code,
        "test:createViewAndComponentInView",
    )?;

    let delete_and_erase_components_code = r#"
    async function main({ thisComponent, currentView }: Input): Promise<Output> {
        const components = thisComponent.properties?.si?.resourceId?.split(",");
        console.log(components);
        const deleteComponent = components[0];
        const deleteComponentWithResource = components[1];
        const deleteComponentStillOnHead = components[2];
        const eraseComponent = components[3]; 

        return {
            status: "ok",
            ops: {
                delete: [
                    deleteComponent,
                    deleteComponentWithResource,
                    deleteComponentStillOnHead
                ],
                erase: [eraseComponent],
            }
        };
    }
    "#;
    let delete_and_erase_components = build_management_func(
        delete_and_erase_components_code,
        "test:deleteAndEraseComponents",
    )?;

    let remove_all_components_from_a_view_and_the_view_code = r#"
    async function main({ thisComponent, components, currentView }: Input): Promise<Output> {
        const viewName = thisComponent.properties?.si?.resourceId ?? currentView;

        return {
            status: "ok",
            ops: {
                views: { remove: [viewName] },
                remove: {
                    [viewName]: Object.values(components).map((component) => component.properties.si.name),
                }
            }
        };
    }
    "#;
    let remove_all_components_from_a_view_and_the_view = build_management_func(
        remove_all_components_from_a_view_and_the_view_code,
        "test:removeAllCompsFromViewAndRemoveView",
    )?;

    let override_values_set_by_sockets_code = r#"
    async function main({ thisComponent, components }: Input): Promise<Output> {
        const thisName = thisComponent.properties?.si?.name ?? "unknown";
        const componentName = `bluey`;

        return {
            status: "ok",
            ops: {
                update: { self: { properties: { si: { type: "configurationFrameDown" } } } },
                create: {
                    [componentName]: {
                        kind: "small odd lego",
                        properties: { 
                            si: { name },
                            domain: {
                                one: `bingo`
                            } 
                        },
                        parent: "self"
                    }
                }
            }
        }
    }
    "#;
    let override_values_set_by_sockets = build_management_func(
        override_values_set_by_sockets_code,
        "test:overrideValuesSetBySocket",
    )?;
    let fn_name = "test:deleteActionSmallLego";
    let delete_action_func = build_action_func(delete_action_code, fn_name)?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldSmallLegoAsset";
    let small_lego_authoring_schema_func = build_asset_func(fn_name)?;

    let bricks = LegoBricks::new_for_odd()?;

    let small_lego_schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(schema_name)
                .build()?,
        )
        .variant(
            SchemaVariantSpec::builder()
                .version("v0")
                .unique_id("small_lego_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&small_lego_authoring_schema_func.unique_id)
                        .component_type(ComponentType::ConfigurationFrameUp)
                        .build()?,
                )
                .domain_prop(bricks.domain_name_prop)
                .domain_prop(bricks.domain_one_prop)
                .domain_prop(bricks.domain_two_prop)
                .domain_prop(
                    PropSpec::builder()
                        .name("test_result")
                        .kind(PropKind::Json)
                        .build()?,
                )
                // Input socket "one"
                .socket(bricks.socket_one)
                // Output socket "two"
                .socket(bricks.socket_two)
                .socket(
                    SocketSpec::builder()
                        .name("arity_one")
                        .data(
                            SocketSpecData::builder()
                                .name("arity_one")
                                .connection_annotations(serde_json::to_string(&vec!["arity_one"])?)
                                .arity(SocketSpecArity::One)
                                .kind(SocketSpecKind::Input)
                                .build()?,
                        )
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Create)
                        .func_unique_id(&create_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Refresh)
                        .func_unique_id(&refresh_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Manual)
                        .func_unique_id(&update_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Destroy)
                        .func_unique_id(&delete_action_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Import small odd lego")
                        .func_unique_id(&import_management_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Clone")
                        .func_unique_id(&clone_me_mgmt_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Update")
                        .func_unique_id(&update_mgmt_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Update in View")
                        .func_unique_id(&update_in_view_mgmt_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Create and Connect From Self")
                        .func_unique_id(&create_and_connect_from_self_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Create and Connect to Self")
                        .func_unique_id(&create_and_connect_to_self_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Create and Connect to Inputs")
                        .func_unique_id(&create_and_connect_to_inputs_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Connect to Inputs")
                        .func_unique_id(&connect_to_inputs_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Disconnect from Inputs")
                        .func_unique_id(&disconnect_from_inputs_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Get Input Values")
                        .func_unique_id(&get_input_values_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Deeply Nested Children")
                        .func_unique_id(&deeply_nested_children.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Create and Connect to Self as Children")
                        .func_unique_id(&create_and_connect_to_self_as_children_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Create in Other Views")
                        .func_unique_id(&create_component_in_other_views.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Create View and Component in View")
                        .func_unique_id(&create_view_and_component_in_view.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Delete and Erase")
                        .func_unique_id(&delete_and_erase_components.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Remove View and Components")
                        .func_unique_id(&remove_all_components_from_a_view_and_the_view.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Override Props")
                        .func_unique_id(&override_values_set_by_sockets.unique_id)
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let small_odd_lego_spec = small_lego_builder
        .func(identity_func_spec)
        .func(refresh_action_func)
        .func(create_action_func)
        .func(update_action_func)
        .func(delete_action_func)
        .func(small_lego_authoring_schema_func)
        .func(import_management_func)
        .func(clone_me_mgmt_func)
        .func(update_mgmt_func)
        .func(update_in_view_mgmt_func)
        .func(create_and_connect_from_self_func)
        .func(create_and_connect_to_self_func)
        .func(create_and_connect_to_self_as_children_func)
        .func(create_and_connect_to_inputs_func)
        .func(connect_to_inputs_func)
        .func(disconnect_from_inputs_func)
        .func(get_input_values_func)
        .func(deeply_nested_children)
        .func(create_component_in_other_views)
        .func(create_view_and_component_in_view)
        .func(delete_and_erase_components)
        .func(remove_all_components_from_a_view_and_the_view)
        .func(override_values_set_by_sockets)
        .schema(small_lego_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(small_odd_lego_spec)?;
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id.into()),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}

pub(crate) async fn migrate_test_exclusive_schema_small_even_lego(
    ctx: &DalContext,
    schema_id: SchemaId,
) -> BuiltinsResult<()> {
    let mut small_lego_builder = PkgSpec::builder();

    let schema_name = "small even lego";

    small_lego_builder
        .name(schema_name)
        .version(PKG_VERSION)
        .created_by(PKG_CREATED_BY);

    let identity_func_spec = create_identity_func()?;

    // Build Create Action Func
    let create_action_code = "async function main() {
                return { payload: { \"poop\": \"created\", \"refresh_count\": 0 }, status: \"ok\" };
            }";

    let fn_name = "test:createActionSmallLego";
    let create_action_func = build_action_func(create_action_code, fn_name)?;

    // Build Refresh Action Func

    let refresh_action_code = r#"async function main(component: Input) {
    const currentCount = typeof component.properties.resource?.payload?.refresh_count === 'number' 
        ? component.properties.resource.payload.refresh_count 
        : 0;
    const newCount = currentCount + 1;

    if (component.properties.resource?.payload?.poop) { 
        return { 
            payload: { 
                "poop": "refreshed again", 
                "refresh_count": newCount 
            }, 
            status: "ok" 
        };
    }
    else {
        return { 
            payload: { 
                "poop": "refreshed", 
                "refresh_count": newCount 
            }, 
            status: "ok" 
        };
    }
}"#;

    let fn_name = "test:refreshActionSmallLego";
    let refresh_action_func = build_action_func(refresh_action_code, fn_name)?;

    let update_action_code = "async function main(component: Input): Promise<Output> {
              return { payload: { \"poop\": updated }, status: \"ok\" };
            }";
    let fn_name = "test:updateActionSmallLego";
    let update_action_func = build_action_func(update_action_code, fn_name)?;

    // Build Delete Action Func
    let delete_action_code = "async function main() {
        return { payload: null, status: \"ok\" };
    }";

    let fn_name = "test:deleteActionSmallLego";
    let delete_action_func = build_action_func(delete_action_code, fn_name)?;

    // Create Scaffold Func
    let fn_name = "test:scaffoldSmallLegoAsset";
    let small_lego_authoring_schema_func = build_asset_func(fn_name)?;

    // This import func mimics our Clover funcs in name so we can test logic related to that
    let import_management_func_code = r#"async function main({ thisComponent }: Input): Promise<Output> {
        const thisProperties = thisComponent.properties;
        return {
            status: 'ok',
            ops: {
                update: {
                    self: {
                        properties: {
                            ...thisProperties,
                            domain: {
                                ...thisProperties.domain,
                                one: 'twostep',
                            },
                        }
                    }
                },
                actions: {
                    self: {
                        remove: ["create"],
                        add: ["refresh"],
                    },
                },
            },
            message: 'hello'
        }
    }"#;
    let import_management_func_name = "Import from AWS";
    let import_management_func =
        build_management_func(import_management_func_code, import_management_func_name)?;

    let bricks = LegoBricks::new_for_even()?;

    let small_lego_schema = SchemaSpec::builder()
        .name(schema_name)
        .data(
            SchemaSpecData::builder()
                .name(schema_name)
                .category("test exclusive")
                .category_name(schema_name)
                .build()?,
        )
        .variant(
            SchemaVariantSpec::builder()
                .version("v0")
                .unique_id("small_even_lego_sv")
                .data(
                    SchemaVariantSpecData::builder()
                        .version("v0")
                        .color("#ffffff")
                        .func_unique_id(&small_lego_authoring_schema_func.unique_id)
                        .component_type(ComponentType::ConfigurationFrameUp)
                        .build()?,
                )
                .domain_prop(bricks.domain_name_prop)
                .domain_prop(bricks.domain_one_prop)
                .domain_prop(bricks.domain_two_prop)
                .socket(bricks.socket_one)
                .socket(bricks.socket_two)
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Create)
                        .func_unique_id(&create_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Refresh)
                        .func_unique_id(&refresh_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Manual)
                        .func_unique_id(&update_action_func.unique_id)
                        .build()?,
                )
                .action_func(
                    ActionFuncSpec::builder()
                        .kind(ActionKind::Destroy)
                        .func_unique_id(&delete_action_func.unique_id)
                        .build()?,
                )
                .management_func(
                    ManagementFuncSpec::builder()
                        .name("Import from AWS")
                        .func_unique_id(&import_management_func.unique_id)
                        .build()?,
                )
                .build()?,
        )
        .build()?;

    let small_lego_spec = small_lego_builder
        .func(identity_func_spec)
        .func(refresh_action_func)
        .func(create_action_func)
        .func(update_action_func)
        .func(delete_action_func)
        .func(import_management_func)
        .func(small_lego_authoring_schema_func)
        .schema(small_lego_schema)
        .build()?;

    let pkg = SiPkg::load_from_spec(small_lego_spec)?;
    import_pkg_from_pkg(
        ctx,
        &pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id.into()),
            ..Default::default()
        }),
    )
    .await?;

    Ok(())
}
