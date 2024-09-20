import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import { runWithTemporaryChangeset } from "../test_helpers.ts";

export default async function create_and_and_apply_across_change_sets(
  sdfApiClient: SdfApiClient,
) {
  return await create_and_and_apply_across_change_sets_inner(sdfApiClient);
}

async function create_and_and_apply_across_change_sets_inner(
  sdf: SdfApiClient,
) {
  console.log("creating changesets with the test component in them...");
  let changeSetIds: { [key: string]: any }[] = [];

  for (let i = 0; i < 100; i++) {
    const startTime = new Date();
    const changeSetName = `API_TEST - ${startTime.toISOString()}`;
    const data = await sdf.call({
      route: "create_change_set",
      body: {
        changeSetName,
      },
    });
    const changeSetId = data.changeSet.id;
    changeSetIds.push(changeSetId);

    const schemaVariants = await getSchemaVariants(sdf, changeSetId);

    const testResourceActionsVariant = await extractSchemaVariant(
      schemaVariants,
      "TestResourceActions",
      "TestResourceActions",
    );
    const testResourceActionsVariantId =
      testResourceActionsVariant.schemaVariantId;
    await createComponent(
      sdf,
      changeSetId,
      testResourceActionsVariantId,
      0,
      0,
    );
  }

  console.log("Done! Running an apply...");

  await sdf.call({
    route: "apply_change_set",
    body: {
      visibility_change_set_pk: changeSetIds[0],
    },
  });
}

function extractSchemaVariant(
  schemaVariants: any[],
  schemaName: string,
  category?: string,
) {
  const variant = schemaVariants.find(
    (sv) =>
      sv.schemaName === schemaName && (!category || sv.category === category),
  );

  const awsRegionVariantId = variant?.schemaVariantId;
  assert(
    awsRegionVariantId,
    `Expected to find ${schemaName} schema and variant`,
  );

  return variant;
}

async function getSchemaVariants(sdf: SdfApiClient, changeSetId: string) {
  const schemaVariants = await sdf.call({
    route: "schema_variants",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
  assert(
    Array.isArray(schemaVariants),
    "List schema variants should return an array",
  );

  return schemaVariants;
}

async function getDiagram(
  sdf: SdfApiClient,
  changeSetId: string,
): Promise<{ components: any[]; edges: any[] }> {
  const diagram = await sdf.call({
    route: "get_diagram",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });

  assert(
    Array.isArray(diagram?.components),
    "Expected components list on the diagram",
  );
  assert(Array.isArray(diagram?.edges), "Expected edges list on the diagram");

  return diagram;
}

async function createComponent(
  sdf: SdfApiClient,
  changeSetId: string,
  schemaVariantId: string,
  x: number,
  y: number,
  parentId?: string,
): Promise<string> {
  const parentArgs = parentId ? { parentId } : {};
  const payload = {
    schemaVariantId,
    x: x.toString(),
    y: y.toString(),
    visibility_change_set_pk: changeSetId,
    workspaceId: sdf.workspaceId,
    ...parentArgs,
  };
  const createResp = await sdf.call({
    route: "create_component",
    body: payload,
  });
  const componentId = createResp?.componentId;
  assert(componentId, "Expected to get a component id after creation");

  // Run side effect calls
  await Promise.all([
    getQualificationSummary(sdf, changeSetId),
    getActions(sdf, changeSetId),
    getFuncs(sdf, changeSetId),
    getPropertyEditor(sdf, changeSetId, componentId),
  ]);

  return componentId;
}

async function getPropertyEditor(
  sdf: SdfApiClient,
  changeSetId: string,
  componentId: string,
) {
  const values = await sdf.call({
    route: "get_property_values",
    routeVars: {
      componentId,
      changeSetId,
    },
  });
  assert(typeof values?.values === "object", "Expected prop values");
  assert(typeof values?.childValues === "object", "Expected prop childValues:");

  const schema = await sdf.call({
    route: "get_property_schema",
    routeVars: {
      componentId,
      changeSetId,
    },
  });
  assert(typeof schema?.rootPropId === "string", "Expected rootPropId");
  assert(typeof schema?.props === "object", "Expected props");
  assert(typeof schema?.childProps === "object", "Expected childProps list");

  return {
    values,
    schema,
  };
}

async function getQualificationSummary(sdf: SdfApiClient, changeSetId: string) {
  return await sdf.call({
    route: "qualification_summary",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
}

async function getActions(sdf: SdfApiClient, changeSetId: string) {
  return await sdf.call({
    route: "action_list",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
}

async function getFuncs(sdf: SdfApiClient, changeSetId: string) {
  return await sdf.call({
    route: "func_list",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
}
