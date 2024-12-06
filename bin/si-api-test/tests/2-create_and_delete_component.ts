import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import { runWithTemporaryChangeset } from "../test_helpers.ts";

export default async function create_and_delete_component(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  if (changeSetId) {
    return await create_and_delete_component_inner(sdfApiClient, changeSetId);
  } else {
    return runWithTemporaryChangeset(
      sdfApiClient,
      create_and_delete_component_inner,
    );
  }
}

async function create_and_delete_component_inner(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  // Get the Schema Variant ID of Generic Frame
  let schemaVariants = await sdfApiClient.call({
    route: "schema_variants",
    routeVars: { changeSetId },
  });

  const newCreateComponentApi = Array.isArray(schemaVariants?.installed);
  if (newCreateComponentApi) {
    schemaVariants = schemaVariants.installed;
  }

  assert(
    Array.isArray(schemaVariants),
    "List schema variants should return an array",
  );
  const genericFrameVariantId = schemaVariants.find((sv) =>
    sv.schemaName === "Generic Frame"
  )?.schemaVariantId;
  assert(
    genericFrameVariantId,
    "Expected to find Generic Frame schema and variant",
  );

  // Create the Component
  const createComponentPayload = {
    schemaVariantId: genericFrameVariantId,
    x: "0",
    y: "0",
    visibility_change_set_pk: changeSetId,
    workspaceId: sdfApiClient.workspaceId,
  };
  if (newCreateComponentApi) {
    createComponentPayload["schemaType"] = "installed";
  }

  const createComponentResp = await sdfApiClient.call({
    route: "create_component",
    body: createComponentPayload,
  });

  const newComponentId = createComponentResp?.componentId;
  assert(newComponentId, "Expected to get a component id after creation");

  // Check that component exists on diagram
  const diagram = await sdfApiClient.call({
    route: "get_diagram",
    routeVars: { changeSetId },
  });
  assert(diagram?.components, "Expected components list on the diagram");
  assert(
    diagram.components.length === 1,
    "Expected a single component on the diagram",
  );
  const createdComponent = diagram.components[0];
  assert(
    createdComponent?.id === newComponentId,
    "Expected diagram component id to match create component API return ID",
  );
  assert(
    createdComponent?.schemaVariantId === genericFrameVariantId,
    "Expected diagram component schema variant id to match generic frame sv id",
  );

  // Delete the Component
  const deleteComponentPayload = {
    componentIds: [newComponentId],
    forceErase: false,
    visibility_change_set_pk: changeSetId,
    workspaceId: sdfApiClient.workspaceId,
  };
  await sdfApiClient.call({
    route: "delete_components",
    body: deleteComponentPayload,
  });

  // Check that component has been removed from diagram
  const diagramAfterDelete = await sdfApiClient.call({
    route: "get_diagram",
    routeVars: { changeSetId },
  });
  assert(
    diagramAfterDelete?.components,
    "Expected components list on the diagram",
  );
  assert(
    diagramAfterDelete.components.length === 0,
    "Expected no components on the diagram",
  );
}
