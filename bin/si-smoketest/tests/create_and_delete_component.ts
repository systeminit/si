import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";

export default async function create_and_delete_component(sdfApiClient: SdfApiClient) {
  const startTime = new Date();
  const changeSetName = `API_TEST create_and_delete_component - ${startTime.toISOString()}`;

  // CREATE CHANGE SET
  const data = await sdfApiClient.createChangeSet(changeSetName);
  assert(typeof data.changeSet === "object", "Expected changeSet in response");
  const changeSet = data.changeSet;
  assert(changeSet?.id, "Expected Change Set id");
  assert(changeSet?.name === changeSetName, `Changeset name should be ${changeSetName}`);
  const changeSetId = changeSet.id;

  // CREATE COMPONENT
  const schemaVariants = await sdfApiClient.listSchemaVariants(changeSetId);
  assert(Array.isArray(schemaVariants), "List schema variants should return an array");
  const genericFrameVariantId = schemaVariants.find((sv) => sv.schemaName === "Generic Frame")?.schemaVariantId;
  assert(genericFrameVariantId, "Expected to find Generic Frame schema and variant");

  // Actually create component
  const createComponentPayload = {
    schemaVariantId: genericFrameVariantId,
    x: "0",
    y: "0",
    visibility_change_set_pk: changeSetId,
    workspaceId: sdfApiClient.workspaceId,
  };
  const createComponentResp = await sdfApiClient.createComponent(createComponentPayload);
  const newComponentId = createComponentResp?.componentId;
  assert(newComponentId, "Expected to get a component id after creation");

  // Check that component exists on diagram
  const diagram = await sdfApiClient.getDiagram(changeSetId);
  assert(diagram?.components, "Expected components list on the diagram");
  assert(diagram.components.length === 1, "Expected a single component on the diagram");
  const createdComponent = diagram.components[0];
  assert(createdComponent?.id === newComponentId, "Expected diagram component id to match create component API return ID");
  assert(createdComponent?.schemaVariantId === genericFrameVariantId, "Expected diagram component schema variant id to match generic frame sv id");

  // DELETE COMPONENT
  const deleteComponentPayload = {
    componentIds: [newComponentId],
    forceErase: false,
    visibility_change_set_pk: changeSetId,
    workspaceId: sdfApiClient.workspaceId,
  };
  await sdfApiClient.deleteComponents(deleteComponentPayload);

  // Check that component has been removed from diagram
  const diagramAfterDelete = await sdfApiClient.getDiagram(changeSetId);
  assert(diagramAfterDelete?.components, "Expected components list on the diagram");
  assert(diagramAfterDelete.components.length === 0, "Expected no components on the diagram");

  // DELETE CHANGE SET
  await sdfApiClient.abandonChangeSet(changeSetId);
}
