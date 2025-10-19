import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import {
  createComponent,
  createComponentPayload,
  eventualMVAssert,
  getActions,
  getViews,
  runWithTemporaryChangeset,
} from "../test_helpers.ts";

export default async function add_action(sdfApiClient: SdfApiClient) {
  return runWithTemporaryChangeset(sdfApiClient, add_action_inner);
}

export async function add_action_inner(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  let data = await getActions(sdfApiClient, changeSetId);
  // Store the original length of actions to verify later
  assert(Array.isArray(data.actions), "Expected actions to be an array");

  const actionOriginalLength = data.actions.length;

  // Get all Schema Variants
  let createComponentBody = await createComponentPayload(
    sdfApiClient,
    changeSetId,
    "AWS::EC2::Instance",
  );

  // Get the views and find the default one
  const views = await getViews(sdfApiClient, changeSetId);
  const defaultView = views.find((v: any) => v.isDefault);
  assert(defaultView, "Expected to find a default view");

  // Create a Component
  const newComponentId = await createComponent(
    sdfApiClient,
    changeSetId,
    defaultView.id,
    createComponentBody,
  );
  assert(newComponentId, "Expected to get a component id after creation");

  await eventualMVAssert(
    sdfApiClient,
    changeSetId,
    "ActionViewList",
    sdfApiClient.workspaceId,
    (mv) => {
      return (
        mv.actions.some(
          (action: any) => action.componentId === newComponentId,
        ) && mv.actions.length === actionOriginalLength + 1
      );
    },
    "No action with the expected componentId found",
  );
}
