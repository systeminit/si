import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";

export default async function create_and_and_apply_across_change_sets(
  sdfApiClient: SdfApiClient,
) {
  return await create_and_and_apply_across_change_sets_inner(sdfApiClient);
}

async function create_and_and_apply_across_change_sets_inner(
  sdf: SdfApiClient,
) {
  sdf.listenForDVUs();
  console.log("cleaning up any existing components...");
  await cleanupHead(sdf);
  console.log("creating changesets with the test component in them...");
  let changeSetIds: string[] = [];
  let err: Error | undefined = undefined;
  try {
    for (let i = 0; i < 10; i++) {
      const changeSetId = await createChangeSet(sdf);
      changeSetIds.push(changeSetId);

      const {
        schemaVariantId: testResourceActionsVariantId,
        newCreateComponentApi,
      } = await getTestSchemaVariantId(sdf, changeSetId);

      await createComponent(
        sdf,
        changeSetId,
        testResourceActionsVariantId,
        0,
        0,
        undefined,
        newCreateComponentApi,
      );
    }

    await sdf.waitForDVUs(2000, 60000);
    console.log("Done! Running an apply...");
    await sdf.call({
      route: "apply_change_set",
      body: {
        visibility_change_set_pk: changeSetIds.pop(),
      },
    });

    await sdf.waitForDVUs(2000, 60000);
  } catch (e) {
    err = e;
  } finally {
    console.log("verifying and cleaning up change sets...");
    for (const changeSetId of changeSetIds) {
      try {
        const diagram = await getDiagram(sdf, changeSetId);
        
        await sdf.call({
          route: "abandon_change_set",
          body: {
            changeSetId,
          },
        });

        assert(
          diagram.components.length === 2,
          `Expected diagram to have two components after apply, found ${diagram.components.length}`,
        );
      } catch (cleanupError) {
        console.warn(`Failed to clean up changeset ${changeSetId}: ${cleanupError.message}`);
      }
    }
    await cleanupHead(sdf);
  }
  if (err) {
    throw err;
  }
}

async function cleanupHead(sdf: SdfApiClient): Promise<void> {
  const cleanupChangeSetId = await createChangeSet(sdf);
  const diagram = await getDiagram(sdf, cleanupChangeSetId);
  const currentComponentIds = diagram.components.map((c) => c.id);
  if (currentComponentIds) {
    const deleteComponentPayload = {
      componentIds: currentComponentIds,
      forceErase: false,
      visibility_change_set_pk: cleanupChangeSetId,
      workspaceId: sdf.workspaceId,
    };
    await sdf.call({
      route: "delete_components",
      body: deleteComponentPayload,
    });
    await sdf.call({
      route: "apply_change_set",
      body: {
        visibility_change_set_pk: cleanupChangeSetId,
      },
    });
  }
}

async function createChangeSet(sdf: SdfApiClient): Promise<string> {
  const startTime = new Date();
  const changeSetName = `API_TEST - 1-create_and_apply_across_change_sets.ts - ${startTime.toISOString()}`;
  const data = await sdf.call({
    route: "create_change_set",
    body: {
      changeSetName,
    },
  });
  return data.changeSet.id;
}

async function getTestSchemaVariantId(
  sdf: SdfApiClient,
  changeSetId: string,
): Promise<{ schemaVariantId: string; newCreateComponentApi: boolean }> {
  const { schemaVariants, newCreateComponentApi } = await getSchemaVariants(
    sdf,
    changeSetId,
  );

  const testResourceActionsVariant = await extractSchemaVariant(
    schemaVariants,
    "TestResourceActions",
    "",
  );
  assert(
    typeof testResourceActionsVariant !== "undefined",
    "TestResourceActions variant should exist",
  );
  return {
    schemaVariantId: testResourceActionsVariant.schemaVariantId,
    newCreateComponentApi,
  };
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

  return variant;
}

async function getSchemaVariants(sdf: SdfApiClient, changeSetId: string) {
  let schemaVariants = await sdf.call({
    route: "schema_variants",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
  const newCreateComponentApi = Array.isArray(schemaVariants?.installed);
  if (newCreateComponentApi) {
    schemaVariants = schemaVariants.installed;
  }

  assert(
    Array.isArray(schemaVariants),
    "List schema variants should return an array",
  );

  return { schemaVariants, newCreateComponentApi };
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
  newCreateComponentApi?: boolean,
): Promise<string> {
  const parentArgs = parentId ? { parentId } : {};
  const payload = {
    schemaType: newCreateComponentApi ? "installed" : undefined,
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
