import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import { createComponent, createComponentPayload, eventualMVAssert, getVariants, getViews, runWithTemporaryChangeset } from "../test_helpers.ts";

export default async function create_and_upgrade_component(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  if (changeSetId) {
    return await create_and_upgrade_component_inner(sdfApiClient, changeSetId);
  } else {
    return runWithTemporaryChangeset(
      sdfApiClient,
      create_and_upgrade_component_inner,
    );
  }
}

async function create_and_upgrade_component_inner(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  // Get all Schema Variants
  let schemaVariants = await getVariants(sdfApiClient, changeSetId);
  let createComponentBody = createComponentPayload(schemaVariants, "Region");

  // Get the views and find the default one
  const views = await getViews(sdfApiClient, changeSetId);
  const defaultView = views.find((v: any) => v.isDefault);
  assert(defaultView, "Expected to find a default view");

  // Create a region component
  const newComponentId = await createComponent(
    sdfApiClient,
    changeSetId,
    defaultView.id,
    createComponentBody,
  );
  assert(newComponentId, "Expected to get a component id after creation");
  const region = await sdfApiClient.mjolnir(changeSetId, "Component", newComponentId);
  assert(region, "Expected to get a region after creation");
  const originalSchemaVariantId = region.schemaVariantId.id;
  assert(originalSchemaVariantId, "Expected to get a schema variant id after creation");
  const schemaId = region.schemaId;
  assert(schemaId, "Expected to get a schema id after creation");

  // ensure this schema is the default variant and there is no editing variant
  const schemaMembers = await sdfApiClient.mjolnir(changeSetId, "SchemaMembers", schemaId);
  assert(schemaMembers, "Expected to get a schema after creation");
  assert(schemaMembers.defaultVariantId === originalSchemaVariantId, "Expected default variant to match original schema variant id");
  assert(schemaMembers.editingVariantId === null, "Expected no editing variant for the schema");

  // unlock the region schema variant 
  const newSchemaVariant = await sdfApiClient.call({
    route: "create_unlocked_copy",
    routeVars: {
      workspaceId: sdfApiClient.workspaceId,
      changeSetId,
      schemaVariantId: originalSchemaVariantId,
    },
  });
  assert(newSchemaVariant, "Expected to get a new schema variant after unlocking");
  console.log("Created new schema variant:", newSchemaVariant);

  // see that the component has an upgrade available
  await eventualMVAssert(
    sdfApiClient,
    changeSetId,
    "SchemaMembers",
    schemaId,
    (mv) => mv.defaultVariantId === originalSchemaVariantId && mv.editingVariantId !== null,
    "Component MV should have upgradeAvailable set to true",
  );
  // upgrade the component
  const upgradeResp = await sdfApiClient.call({
    route: "upgrade",
    routeVars: {
      workspaceId: sdfApiClient.workspaceId,
      changeSetId,
    },
    body: {
      componentIds: [newComponentId],
    },
  });
  // see that the upgrade succeeds
  await eventualMVAssert(
    sdfApiClient,
    changeSetId,
    "Component",
    newComponentId,
    (mv) => mv.schemaVariantId !== originalSchemaVariantId,
    "Component MV should have been upgraded",
  );
}
