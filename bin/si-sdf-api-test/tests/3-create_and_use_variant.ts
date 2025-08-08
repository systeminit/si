import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import { runWithTemporaryChangeset } from "../test_helpers.ts";

export default async function create_variant(sdfApiClient: SdfApiClient) {
  return runWithTemporaryChangeset(sdfApiClient, create_variant_inner);
}

export async function create_variant_inner(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  const startTime = new Date();
  const variantName = `Test_Variant - ${startTime.toISOString()}`;

  const newCreateComponentApi = Array.isArray((await sdfApiClient.call({
    route: "schema_variants",
    routeVars: {
      workspaceId: sdfApiClient.workspaceId,
      changeSetId,
    },
  }))?.installed);

  // Create the Varint
  const createVariantPayload = {
    "name": variantName,
    "color": "0",
    "visibility_change_set_pk": changeSetId,
  };

  const createVariantResp = await sdfApiClient.call({
    route: "create_variant",
    routeVars: {
      workspaceId: sdfApiClient.workspaceId,
    },
    body: createVariantPayload,
  });

  const schemaVariantId = createVariantResp?.schemaVariantId;
  assert(schemaVariantId, "Expected to get a schemaVariantId after creation");

  // create new variant as a component
  let createInstancePayload = {
    "schemaVariantId": schemaVariantId,
    "x": "200",
    "y": "0",
    "visibility_change_set_pk": changeSetId,
    "workspaceId": sdfApiClient.workspaceId,
  };
  if (newCreateComponentApi) {
    createInstancePayload["schemaType"] = "installed";
  }

  const createInstanceResp = await sdfApiClient.call({
    route: "create_component",
    body: createInstancePayload,
  });

  const newSchemaComponentId = createInstanceResp?.componentId;
  assert(newSchemaComponentId, "Expected to get a component id after creation");

  // Check that components exists on diagram
  const diagram = await sdfApiClient.call({
    route: "get_diagram",
    routeVars: {
      workspaceId: sdfApiClient.workspaceId,
      changeSetId,
    },
  });

  assert(diagram?.components, "Expected components list on the diagram");
  assert(
    diagram.components.length === 1,
    "Expected a single component on the diagram",
  );

  const regionComponentOnDiagram = diagram.components.find((c) =>
    c.id === newSchemaComponentId
  );
  assert(
    regionComponentOnDiagram,
    "Expected to find the new schema variant on the diagram",
  );
  assert(
    regionComponentOnDiagram?.schemaVariantId === schemaVariantId,
    "Expected diagram component schema variant id to be correct",
  );
}
