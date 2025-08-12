import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import { eventualMVAssert, getVariants, getChangeSetIndex, runWithTemporaryChangeset, createComponentPayload, getViews } from "../test_helpers.ts";

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
    sdfApiClient: SdfApiClient,
    changeSetId: string,
) {

    // TODO: for these tests might be nice to have this happen in runWithTemporaryChangeset
    await sdfApiClient.fetchChangeSetIndex(changeSetId);
    // Get the schema variants (uninstalled and installed)
    let schemaVariants = await getVariants(sdfApiClient, changeSetId);
    let createComponentBody = createComponentPayload(schemaVariants, SCHEMA_NAME);

    // Get the views and find the default one
    const views = await getViews(sdfApiClient, changeSetId);
    const defaultView = views.find((v: any) => v.isDefault);
    assert(defaultView, "Expected to find a default view");

    // Create a Component
    const createComponentResp = await sdfApiClient.call({
        route: "create_component_v2",
        routeVars: {
            workspaceId: sdfApiClient.workspaceId,
            changeSetId,
            viewId: defaultView.id,
        },
        body: createComponentBody,
    });
    const { newComponentId, newComponentName } = { newComponentId: createComponentResp?.componentId, newComponentName: createComponentResp?.materializedView?.name };
    assert(newComponentId, "Expected to get a component id after creation");

    // Wait for and verify component MV
    await eventualMVAssert(
        sdfApiClient,
        changeSetId,
        "Component",
        newComponentId,
        (mv) => mv.name === newComponentName,
        "Component MV should exist and have matching name"
    );

    
    // Delete the Component
    const deleteComponentPayload = {
        componentIds: [newComponentId],
        forceErase: false,
    };
    await sdfApiClient.call({
        route: "delete_components_v2",
        routeVars: {
            workspaceId: sdfApiClient.workspaceId,
            changeSetId,
        },
        body: deleteComponentPayload,
    });

    // Verify component deletion
    await eventualMVAssert(
        sdfApiClient,
        changeSetId,
        "ComponentList",
        sdfApiClient.workspaceId,
        (mv) => mv.components.length === 0,
        "Diagram should have no components after deletion"
    );

    // await an installed schema variant now
    await eventualMVAssert(
        sdfApiClient,
        changeSetId,
        "SchemaVariantCategories",
        sdfApiClient.workspaceId,
        (mv) => mv.categories.some((c: any) => c.schemaVariants.some((v: any) => v.type === "installed" && v.name === SCHEMA_NAME)),
        "Schema variant should be installed"
    );
    // create another one
    schemaVariants = await getVariants(sdfApiClient, changeSetId);
    createComponentBody = createComponentPayload(schemaVariants, SCHEMA_NAME);
    const createComponent2Resp = await sdfApiClient.call({
        route: "create_component_v2",
        routeVars: {
            workspaceId: sdfApiClient.workspaceId,
            changeSetId,
            viewId: defaultView.id,
        },
        body: createComponentBody,
    });
    const { newComponentId2, newComponentName2 } = { newComponentId2: createComponent2Resp?.componentId, newComponentName2: createComponent2Resp?.materializedView?.name };
    assert(newComponentId2, "Expected to get a component id after creation");
    await eventualMVAssert(
        sdfApiClient,
        changeSetId,
        "Component",
        newComponentId2,
        (mv) => mv.name === newComponentName2,
        "Component MV should exist and have matching name"
    );

}


