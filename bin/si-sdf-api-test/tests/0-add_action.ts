import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import { runWithTemporaryChangeset } from "../test_helpers.ts";

export default async function add_action(sdfApiClient: SdfApiClient) {
  return runWithTemporaryChangeset(sdfApiClient, add_action_inner);
}

export async function add_action_inner(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  let data = await sdfApiClient.call({
    route: "action_list",
    routeVars: {
      changeSetId,
    },
  });

  const actionOriginalLength = data.length;

  // Get all Schema Variants
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
  const EC2InstanceVariantId = schemaVariants.find(
    (sv) => sv.schemaName === "EC2 Instance",
  )?.schemaVariantId;
  assert(
    EC2InstanceVariantId,
    "Expected to find EC2 Instance schema and variant",
  );

  // Create the Component
  const createComponentPayload = {
    schemaVariantId: EC2InstanceVariantId,
    x: "0",
    y: "0",
    visibility_change_set_pk: changeSetId,
    workspaceId: sdfApiClient.workspaceId,
  };
  if (newCreateComponentApi) {
    createComponentPayload["schemaType"] = 'installed';
  }

  const createComponentResp = await sdfApiClient.call({
    route: "create_component",
    body: createComponentPayload,
  });

  const newComponentId = createComponentResp?.componentId;
  assert(newComponentId, "Expected to get a component id after creation");

  // Assert that an action has been queued for EC2 Instance component
  data = await sdfApiClient.call({
    route: "action_list",
    routeVars: {
      changeSetId,
    },
  });

  // Check if any object in the array has the expected componentId
  const hasComponentId = data.some(
    (item) => item.componentId === newComponentId,
  );

  assert.strictEqual(
    hasComponentId,
    true,
    "No action with the expected componentId found",
  );
  assert.strictEqual(
    data.length,
    actionOriginalLength + 1,
    "Incorrect number of actions listed",
  );
}
