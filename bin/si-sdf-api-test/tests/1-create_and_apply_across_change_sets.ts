import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import { abandon_all_change_sets, createComponent, createComponentPayload, eventualMVAssert, getVariants, getViews, installModule } from "../test_helpers.ts";

export default async function create_and_and_apply_across_change_sets(
  sdfApiClient: SdfApiClient,
) {
  return await create_and_and_apply_across_change_sets_inner(sdfApiClient);
}
const TEST_RESOURCE_ACTION_MODULE_ID = "01JMJB1QW3M8D7MNN8NGBR739J";
async function create_and_and_apply_across_change_sets_inner(
  sdf: SdfApiClient,
) {
  sdf.listenForDVUs();
  console.log("cleaning up any existing components and open change sets...");
  await cleanupHead(sdf);
  console.log("creating changesets with the test component in them...");
  let changeSetIds: string[] = [];
  let err: Error | undefined = undefined;

  try {
    // Create 10 change sets, each with one test component in them
    for (let i = 0; i < 10; i++) {
      const changeSetId = await createChangeSet(sdf);
      changeSetIds.push(changeSetId);
      const views = await getViews(sdf, changeSetId);
      const defaultView = views.find((v: any) => v.isDefault);
      assert(defaultView, "Expected to find a default view");


      let schemaVariants = await getVariants(sdf, changeSetId);

      // first let's make sure our test asset is installed
      // this is really just so we can run this against any workspace
      if (!schemaVariants.installed.some((v: any) => v.schemaName === "TestResourceActions")) {
        const installResult = await installModule(sdf, changeSetId, TEST_RESOURCE_ACTION_MODULE_ID);
        console.log("Install Result:", installResult);
        assert(installResult[0]?.schemaVariantId, "Expected to get a schemaVariantId after installing TestResourceActions module");
        const installedVariantId = installResult[0].schemaVariantId;
        console.log("Installing TestResourceActions schema variant...", installResult);
        await eventualMVAssert(
          sdf,
          changeSetId, "SchemaVariantCategories",
          sdf.workspaceId,
          (mv) => {
            const installedList: string[] = mv.categories.flatMap((c: any) => c.schemaVariants.filter((v: any) => v.type === "installed").map((v: any) => {
              return v.id;
            }));
            console.log("Installed Schema Variants:", installedList);
            console.log("Expected Schema Variant ID:", installedVariantId);
            return installedList.some((v: any) => v === installedVariantId);
          },
          "Expected TestResourceActions schema variant to be installed",
        );
        schemaVariants = await getVariants(sdf, changeSetId);
      }

      // Create a component with the TestResourceActions schema variant
      console.log("Creating a component with the TestResourceActions schema variant...");
      let createComponentBody = createComponentPayload(schemaVariants, "TestResourceActions");
      const newComponentId = await createComponent(sdf, changeSetId, defaultView.id, createComponentBody);
      assert(newComponentId, "Expected to get a component id after creation");
      await sdf.waitForDVURoots(changeSetId, 2000, 60000);
      await eventualMVAssert(sdf, changeSetId, "ComponentList", sdf.workspaceId, (mv) => mv.components.length === 1, "Expected one component in the ComponentList MV after apply", 60000);

    }

    console.log("Done! Running an apply...");
    const workspaceId = sdf.workspaceId;
    const appliedChangeSetId = changeSetIds.pop() || "";
    await sdf.call({
      route: "apply",
      routeVars: {
        workspaceId,
        changeSetId: appliedChangeSetId,
      },
    });

  } catch (e) {
    console.error("An error occurred during the test:", e);
    err = e;
  } finally {
    console.log("verifying and cleaning up change sets...");
    for (const changeSetId of changeSetIds) {
      try {
        await eventualMVAssert(sdf, changeSetId, "ComponentList", sdf.workspaceId, (mv) => mv.components.length === 2, "Expected one component in the ComponentList MV after apply", 60000);

        await sdf.call({
          route: "abandon_change_set",
          body: {
            changeSetId,
          },
        });
      } catch (cleanupError) {
        console.warn(
          `Failed to clean up changeset ${changeSetId}: ${cleanupError.message}`,
        );
      }
    }
    await cleanupHead(sdf);
  }
  if (err) {
    throw err;
  }
}

async function cleanupHead(sdf: SdfApiClient): Promise<void> {
  const changeSetId = await createChangeSet(sdf);
  const workspaceId = sdf.workspaceId;

  const components = await sdf.mjolnir(changeSetId, "ComponentList", workspaceId);

  const currentComponentIds = components.components.map((c) => c.id);
  if (currentComponentIds) {
    const deleteComponentPayload = {
      componentIds: currentComponentIds,
      forceErase: false,
    };
    await sdf.call({
      route: "delete_components_v2",
      routeVars: {
        workspaceId: sdf.workspaceId,
        changeSetId,
      },
      body: deleteComponentPayload,
    });
    await sdf.call({
      route: "apply",
      routeVars: {
        workspaceId,
        changeSetId,
      },
    });
  }
  // also abandon all open change sets in case there are any leftover
  await abandon_all_change_sets(sdf);
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
  await sdf.fetchChangeSetIndex(data.changeSet.id);
  return data.changeSet.id;
}
