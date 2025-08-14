import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import { eventualMVAssert, getVariants, runWithTemporaryChangeset, createComponentPayload, getViews } from "../test_helpers.ts";

// ==========================
// Tunables for testing here!
const SCHEMA_NAME = "AWS::EC2::KeyPair"; // the schema name to be used for the test
// ==========================

export default async function check_mjolnir(
    sdfApiClient: SdfApiClient,
    changeSetId: string,
) {
    if (changeSetId) {
        return await check_mjolnir_inner(sdfApiClient, changeSetId);
    } else {
        return runWithTemporaryChangeset(sdfApiClient, check_mjolnir_inner);
    }
}

async function check_mjolnir_inner(
    sdf: SdfApiClient,
    changeSetId: string,
) {

    // Get the schema variants (uninstalled and installed)
    let schemaVariants = await getVariants(sdf, changeSetId);
    let createComponentBody = createComponentPayload(schemaVariants, SCHEMA_NAME);

    // Get the views and find the default one
    const views = await getViews(sdf, changeSetId);
    const defaultView = views.find((v: any) => v.isDefault);
    assert(defaultView, "Expected to find a default view");

    // Create a Component
    const createComponentResp = await sdf.call({
        route: "create_component_v2",
        routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
            viewId: defaultView.id,
        },
        body: createComponentBody,
    });
    const { newComponentId, newComponentName } = { newComponentId: createComponentResp?.componentId, newComponentName: createComponentResp?.materializedView?.name };
    assert(newComponentId, "Expected to get a component id after creation");

    // Wait for and verify component MV
    await eventualMVAssert(
        sdf,
        changeSetId,
        "Component",
        newComponentId,
        (mv) => mv.name === newComponentName,
        "Component MV should exist and have matching name"
    );
    const componentMV = await sdf.mjolnir(changeSetId, "Component", newComponentId);
    assert(componentMV, "Expected to get a component schema variant id after creation");
    const schemaVariantId = componentMV.schemaVariantId.id;
    // Delete the Component
    const deleteComponentPayload = {
        componentIds: [newComponentId],
        forceErase: false,
    };
    await sdf.call({
        route: "delete_components_v2",
        routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
        },
        body: deleteComponentPayload,
    });

    // Verify component deletion
    await eventualMVAssert(
        sdf,
        changeSetId,
        "ComponentList",
        sdf.workspaceId,
        (mv) => mv.components.length === 0,
        "Should have no components after deletion"
    );

    // await an installed schema variant now
    await eventualMVAssert(
        sdf,
        changeSetId,
        "SchemaVariantCategories",
        sdf.workspaceId,
        (mv) => mv.categories.some((c: any) => c.schemaVariants.some((v: any) => {
            if(v.type === "installed"){
                console.log("Found installed schema variant:", v.id, "Expected:", schemaVariantId);
            }
            return v.type === "installed" && v.id === schemaVariantId;
            })),
        "Schema variant should be installed",
    );
    // create another one
    schemaVariants = await getVariants(sdf, changeSetId);
    createComponentBody = createComponentPayload(schemaVariants, SCHEMA_NAME);
    const createComponent2Resp = await sdf.call({
        route: "create_component_v2",
        routeVars: {
            workspaceId: sdf.workspaceId,
            changeSetId,
            viewId: defaultView.id,
        },
        body: createComponentBody,
    });
    const { newComponentId2, newComponentName2 } = { newComponentId2: createComponent2Resp?.componentId, newComponentName2: createComponent2Resp?.materializedView?.name };
    assert(newComponentId2, "Expected to get a component id after creation");
    await eventualMVAssert(
        sdf,
        changeSetId,
        "Component",
        newComponentId2,
        (mv) => mv.name === newComponentName2,
        "Component MV should exist and have matching name"
    );

}


