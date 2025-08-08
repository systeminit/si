import assert from "node:assert";
import { SdfApiClient } from "../sdf_api_client.ts";
import { runWithTemporaryChangeset } from "../test_helpers.ts";

// ==========================
// Tunables for testing here!
const SCHEMA_NAME = "AWS::EC2::KeyPair"; // the schema name to be used for the test (needs to be installed on HEAD in the target workspace)
const INDEX_MAX_CALLS = 20; // the max number of times the index route can be called
const INDEX_SLEEP_MS = 10; // the sleep duration between each index route call
const MJOLNIR_MAX_CALLS = 20; // the max number of times the mjolnir route can be called
const MJOLNIR_SLEEP_MS = 10; // the sleep duration between each mjolnir route call
// ==========================

export default async function check_mjolnir(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  if (changeSetId) {
    return await check_mjolnir_inner(sdfApiClient, changeSetId);
  } else {
    return runWithTemporaryChangeset(
      sdfApiClient,
      check_mjolnir_inner,
    );
  }
}

async function check_mjolnir_inner(
  sdfApiClient: SdfApiClient,
  changeSetId: string,
) {
  // Get the schema variant
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
  const schemaName = SCHEMA_NAME;
  const schemaVariantId = schemaVariants.find(
    (sv) => sv.schemaName === schemaName,
  )?.schemaVariantId;
  assert(schemaVariantId, `Expected to find ${schemaName} schema and variant`);

  // Create the Component
  const createComponentPayload = {
    schemaVariantId,
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
    createdComponent?.schemaVariantId === schemaVariantId,
    "Expected diagram component schema variant id to match sv id",
  );

  // Get the checksum for the individual component MV
  let maybeChecksum = null;
  for (let count = 1; count < INDEX_MAX_CALLS - 1; count++) {
    const response = await sdfApiClient.call({
      route: "index",
      routeVars: { changeSetId },
    });

    // If we find the checksum, then we are done
    for (const mv of response?.frontEndObject?.data?.mvList) {
      if (mv?.id === newComponentId && mv?.kind === "Component") {
        maybeChecksum = mv?.checksum;
        break;
      }
    }
    if (maybeChecksum) {
      break;
    }

    console.log(
      `index not yet built or available (sleeping for ${INDEX_SLEEP_MS}ms: attempt ${count} of ${INDEX_MAX_CALLS})`,
    );
    await new Promise((f) => setTimeout(f, INDEX_SLEEP_MS));
  }

  // Confirm that we have the checksum
  const checksum = maybeChecksum!;

  // With the checksum in hand, get the component MV
  let maybeComponentName = null;
  for (let count = 1; count < MJOLNIR_MAX_CALLS - 1; count++) {
    const response = await sdfApiClient.call(
      {
        route: "mjolnir",
        routeVars: {
          changeSetId,
          materializedViewId: newComponentId,
          referenceKind: "Component",
          materializedViewChecksum: checksum,
        },
      },
      true,
    );

    // If the call succeeded, try to get the component name
    if (response?.status === 200) {
      try {
        const json = await response.json();
        maybeComponentName = json?.frontEndObject?.data?.name;
      } catch (err) {
        console.error("Error trying to parse response body as JSON", err);
      }

      // If we found the component name, then we are done (otherwise, retry!)
      if (maybeComponentName) {
        break;
      }
    } else if (response?.status === 404) {
      // Retry on 404
      console.log(`Received 404 error (retrying): ${await response.text()}`);
    } else {
      // Fail on non-200 and non-404 errors
      throw new Error(`Error ${response.status}: ${await response.text()}`);
    }

    console.log(
      `mjolnir not yet available (sleeping for ${MJOLNIR_SLEEP_MS}ms: attempt ${count} of ${MJOLNIR_MAX_CALLS})`,
    );
    await new Promise((f) => setTimeout(f, MJOLNIR_SLEEP_MS));
  }

  // Make sure the MV data looks as we expect
  assert(
    createdComponent?.displayName === maybeComponentName,
    "Expected diagram component name to match component MV name",
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
