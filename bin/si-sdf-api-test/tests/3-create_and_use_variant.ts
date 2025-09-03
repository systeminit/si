import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import {
  createComponent,
  eventualMVAssert,
  getViews,
  runWithTemporaryChangeset,
  sleep,
  sleepBetween,
} from "../test_helpers.ts";

export default async function create_variant(sdfApiClient: SdfApiClient) {
  return runWithTemporaryChangeset(sdfApiClient, create_variant_inner);
}

export async function create_variant_inner(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  // Get the views and find the default one
  const views = await getViews(sdfApiClient, changeSetId);
  const defaultView = views.find((v: any) => v.isDefault);
  assert(defaultView, "Expected to find a default view");

  const startTime = new Date();
  const variantName = `Test_Variant - ${startTime.toISOString()}`;

  // Create the Variant
  const createVariantPayload = {
    name: variantName,
    color: "0",
    visibility_change_set_pk: changeSetId,
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
  await eventualMVAssert(
    sdfApiClient,
    changeSetId,
    "SchemaVariant",
    schemaVariantId,
    (mv) => mv.id === schemaVariantId,
    "SchemaVariant MV should exist and have matching id",
    180000, // give it 120 seconds to appear as we'll have to rebuild SchemaVariantCategories as well
  );
  // create new variant as a component
  let createInstancePayload = {
    schemaVariantId: schemaVariantId,
    x: "0",
    y: "0",
    height: "0",
    width: "0",
    parentId: null,
    schemaType: "installed",
  };
  const newSchemaComponentId = await createComponent(
    sdfApiClient,
    changeSetId,
    defaultView.id,
    createInstancePayload,
  );
  assert(newSchemaComponentId, "Expected to get a component id after creation");

  await sleepBetween(4000, 10000);

  // make sure component is in the list
  await eventualMVAssert(
    sdfApiClient,
    changeSetId,
    "ComponentList",
    sdfApiClient.workspaceId,
    (mv) =>
      mv.components.some((c: any) => c.id === newSchemaComponentId) &&
      mv.components.length === 1,
    "ComponentList MV should include the new component",
  );
}
