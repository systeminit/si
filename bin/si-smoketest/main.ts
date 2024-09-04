import assert from "node:assert";
import JWT from "npm:jsonwebtoken";
import { createPrivateKey } from "node:crypto";
import { SdfApiClient } from "./sdf_api_client.ts";

if (import.meta.main) {
  assert(Deno.env.get("SDF_API_URL")?.length > 0, "Expected SDF_API_URL env var");
  assert(Deno.env.get("AUTH_API_URL")?.length > 0, "Expected AUTH_API_URL env var");

  assert([2, 3].includes(Deno.args.length), "Expected args: workspaceId, userEmail and userPassword (optional)");
  const workspaceId = Deno.args[0];
  const userId = Deno.args[1];
  const password = Deno.args[2];
  const sdf = await SdfApiClient.init(
    workspaceId,
    userId,
    password
  );

  // Run tests
  // TODO make the tests run in parallel, be filterable, etc. For now this is good enough

  // TODO automatically time tests, we should probably be using a testing library for this anyway
  let testStart = new Date();
  await get_head_changeset(sdf);
  console.log(`get_head_changeset OK - ${(new Date() - testStart)}ms`);

  testStart = new Date();
  await create_and_delete_component(sdf);
  console.log(`create_and_delete_component OK - ${(new Date() - testStart)}ms`);

  console.log("~~ SUCCESS ~~");
}

async function get_head_changeset(sdf: SdfApiClient) {
  const resp = await sdf.fetch("/change_set/list_open_change_sets");
  const data = await resp.json();

  assert(data.headChangeSetId, "Expected headChangeSetId");
  const head = data.changeSets.find((c) => c.id === data.headChangeSetId);
  assert(head, "Expected a HEAD changeset");
}

async function create_and_delete_component(sdf: SdfApiClient) {
  const startTime = new Date();
  const changeSetName = `API_TEST create_and_delete_component - ${startTime.toISOString()}`;

  // CREATE CHANGE SET
  const createChangesetResp = await sdf.fetch("/change_set/create_change_set", {
    method: "POST",
    body: {
      changeSetName
    }
  });
  const data = await createChangesetResp.json();

  assert(typeof data.changeSet === "object", "Expected changeSet in response");
  const changeSet = data.changeSet;
  assert(changeSet?.id, "Expected Change Set id");
  assert(changeSet?.name === changeSetName, `Changeset name should be ${changeSetName}`);
  const changeSetId = changeSet.id;

  // CREATE COMPONENT
  // get schema variant id
  const schemaVariantsResp = await sdf.fetch(`/v2/workspaces/${sdf.workspaceId}/change-sets/${changeSetId}/schema-variants`);
  const schemaVariants = await schemaVariantsResp.json();
  assert(Array.isArray(schemaVariants), "List schema variants should return an array");
  const genericFrameVariantId = schemaVariants.find((sv) => sv.schemaName === "Generic Frame")?.schemaVariantId;
  assert(genericFrameVariantId, "Expected to find Generic Frame schema and variant");


  // actually create component
  let createComponentPayload = {
    "schemaVariantId": genericFrameVariantId,
    "x": "0",
    "y": "0",
    "visibility_change_set_pk": changeSetId,
    "workspaceId": sdf.workspaceId
  };
  const createComponentResp = await sdf.fetch("/diagram/create_component", {
    method: "POST",
    body: createComponentPayload
  });
  const newComponentId = (await createComponentResp.json())?.componentId;
  assert(newComponentId, "Expected to get a component id after creation");

  // Check that component exists on diagram
  const getDiagramResponse = await sdf.fetch(`/diagram/get_diagram?visibility_change_set_pk=${changeSetId}&workspaceId=${sdf.workspaceId}`);
  const diagram = await getDiagramResponse.json();

  assert(diagram?.components, "Expected components list on the diagram");
  assert(diagram.components.length === 1, "Expected a single component on the diagram");
  const createdComponent = diagram.components[0];
  assert(createdComponent?.id === newComponentId, "Expected diagram component id to match create component API return ID");
  assert(createdComponent?.schemaVariantId === genericFrameVariantId, "Expected diagram component schema variant id to match generic frame sv id");


  // DELETE COMPONENT
  let deleteComponentPayload = {
    "componentIds": [newComponentId],
    "forceErase": false,
    "visibility_change_set_pk": changeSetId,
    "workspaceId": sdf.workspaceId
  };
  await sdf.fetch("/diagram/delete_components", {
    method: "POST",
    body: deleteComponentPayload
  });

  // Check that component has been removed from diagram
  const getDiagramAgainResponse = await sdf.fetch(`/diagram/get_diagram?visibility_change_set_pk=${changeSetId}&workspaceId=${sdf.workspaceId}`);
  const diagramAfterDelete = await getDiagramAgainResponse.json();

  assert(diagramAfterDelete?.components, "Expected components list on the diagram");
  assert(diagramAfterDelete.components.length === 0, "Expected a no components on the diagram");


  // DELETE CHANGE SET
  await sdf.fetch("/change_set/abandon_change_set", {
    method: "POST",
    body: {
      changeSetId
    }
  });
}


