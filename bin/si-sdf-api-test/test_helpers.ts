import { SdfApiClient } from "./sdf_api_client.ts";
import assert from "node:assert";

export function sleep(ms: number) {
  const natural_ms = Math.max(0, Math.floor(ms));
  console.log(`Sleeping for ${natural_ms} ms`);
  return new Promise((resolve) => setTimeout(resolve, natural_ms));
}

export function sleepBetween(minMs: number, maxMs: number) {
  if (maxMs < minMs) maxMs = minMs;
  return sleep(minMs + Math.floor((maxMs - minMs) * Math.random()));
}

export async function retryUntil(
  fn: () => Promise<void>,
  timeoutMs: number,
  message?: string,
  retries = 20,
  backoffFactor = 1.2, // exponential backoff factor
  initialDelay = 0.4, // in seconds
): Promise<void> {
  const retryPromise = retryWithBackoff(
    fn,
    retries,
    backoffFactor,
    initialDelay,
  );

  const timeoutPromise = new Promise<void>((_, reject) => {
    setTimeout(() => {
      reject(
        new Error(
          `Timeout after ${timeoutMs}ms while retrying operation: ${message}`,
        ),
      );
    }, timeoutMs);
  });

  return Promise.race([retryPromise, timeoutPromise]);
}

// Run fn n times, with increasing intervals between tries
export async function retryWithBackoff(
  fn: () => Promise<void>,
  retries = 60,
  backoffFactor = 1.2,
  initialDelay = 2, /// in seconds
) {
  const maxDelay = 10;
  let latest_err;
  let try_count = 0;
  let delay = initialDelay;

  console.log("Running retry_with_backoff block");
  do {
    try_count++;
    latest_err = undefined;
    console.log(`try number ${try_count}`);

    try {
      await fn();
    } catch (e) {
      latest_err = e;
      await sleep(delay * 1000);
      delay = Math.min(delay * backoffFactor, maxDelay);
    }
  } while (latest_err && try_count < retries);

  if (latest_err) {
    throw latest_err;
  }
}

export async function runWithTemporaryChangeset(
  sdf: SdfApiClient,
  fn: (sdf: SdfApiClient, changesetId: string) => Promise<void>,
) {
  // CREATE CHANGESET
  const startTime = new Date();
  const changeSetName = `API_TEST - ${fn.name} - ${startTime.toISOString()}`;

  const data = await sdf.call({
    route: "create_change_set",
    body: {
      name: changeSetName,
    },
  });
  assert(typeof data === "object", "Expected changeSet in response");
  const changeSet = data;
  assert(changeSet?.id, "Expected Change Set id");
  assert(
    changeSet?.name === changeSetName,
    `Changeset name should be ${changeSetName}`,
  );
  const changeSetId = changeSet.id;
  console.log(`Created temporary changeset ${changeSetId}`);
  await sleepBetween(1000, 5000);
  console.log("Fetching changeset index before running fn");
  // FETCH CHANGESET INDEX before running fn so we ensure the index has been built
  await sdf.fetchChangeSetIndex(changeSetId);
  // RUN FN
  let err;
  try {
    await fn(sdf, changeSetId);
  } catch (e) {
    err = e;
  }

  // DELETE CHANGE SET
  await sdf.call({
    route: "abandon_change_set",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });

  if (err) {
    throw err;
  }
}

export const nilId = "00000000000000000000000000";

// Various Helpers ------------------------------------------------------------

export async function getFuncs(sdf: SdfApiClient, changeSetId: string) {
  return await sdf.call({
    route: "func_list",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
}

export async function installModule(
  sdf: SdfApiClient,
  changeSetId: string,
  moduleId: string,
) {
  const installModulePayload = {
    visibility_change_set_pk: changeSetId,
    ids: [moduleId],
  };

  return await sdf.call({
    route: "install_module",
    body: installModulePayload,
  });
}

// Schema Variant Helpers ------------------------------------------------------------

export async function createAsset(
  sdf: SdfApiClient,
  changeSetId: string,
  name: string,
): Promise<string> {
  const createAssetPayload = {
    visibility_change_set_pk: changeSetId,
    name,
    color: "#AAFF00",
  };

  const createResp = await sdf.call({
    route: "create_variant",
    body: createAssetPayload,
  });
  const schemaVariantId = createResp?.schemaVariantId;
  assert(schemaVariantId, "Expected to get a schema variant id after creation");
  return schemaVariantId;
}

export async function updateAssetCode(
  sdf: SdfApiClient,
  changeSetId: string,
  schemaVariantId: string,
  newCode: string,
): Promise<string> {
  // Get variant
  const variant = await sdf.call({
    route: "get_variant",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      schemaVariantId,
    },
  });
  // update variant
  const updateVariantBody = {
    visibility_change_set_pk: changeSetId,
    code: newCode,
    variant,
  };

  const saveResp = await sdf.call({
    route: "save_variant",
    body: updateVariantBody,
  });
  const success = saveResp.success;
  assert(success, "save was successful");

  const regenBody = {
    visibility_change_set_pk: changeSetId,
    variant,
  };

  const regenerateResp = await sdf.call({
    route: "regenerate_variant",
    body: regenBody,
  });

  const maybeNewId = regenerateResp.schemaVariantId;
  assert(maybeNewId, "Expected to get a schema variant id after regenerate");
  return maybeNewId;
}

// Component Helpers ------------------------------------------------------------

export async function createComponent(
  sdf: SdfApiClient,
  changeSetId: string,
  viewId: string,
  request: any,
): Promise<string> {
  // Create a Component
  const createComponentResp = await sdf.call({
    route: "create_component_v2",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      viewId: viewId,
    },
    body: request,
  });
  const { newComponentId, newComponentName } = {
    newComponentId: createComponentResp?.componentId,
    newComponentName: createComponentResp?.materializedView?.name,
  };
  assert(newComponentId, "Expected to get a component id after creation");

  // Wait for and verify component MV
  await eventualMVAssert(
    sdf,
    changeSetId,
    "Component",
    newComponentId,
    (mv) => mv.name === newComponentName,
    "Component MV should exist and have matching name",
  );

  return newComponentId;
}

// Func Helpers ------------------------------------------------------------

export async function getFuncRun(
  sdf: SdfApiClient,
  changeSetId: string,
  funcRunId: string,
) {
  console.log(funcRunId);
  const funcRun = await sdf.call({
    route: "get_func_run",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      funcRunId,
    },
  });
  return funcRun;
}
export async function testExecuteFunc(
  sdf: SdfApiClient,
  changeSetId: string,
  funcId: string,
  componentId: string,
  code: string,
  args: any,
) {
  const testExecuteFuncPayload = {
    args,
    code,
    componentId,
  };
  const createFuncResp = await sdf.call({
    route: "test_execute",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      funcId,
    },
    body: testExecuteFuncPayload,
  });
  // returns the func_run_id
  return createFuncResp;
}

export async function createQualification(
  sdf: SdfApiClient,
  changeSetId: string,
  name: string,
  schemaVariantId: string,
  code: string,
) {
  const createFuncPayload = {
    name,
    displayName: name,
    description: "",
    binding: {
      funcId: nilId,
      schemaVariantId,
      bindingKind: "qualification",
      inputs: [],
    },
    kind: "Qualification",
    requestUlid: changeSetId,
  };
  const createFuncResp = await sdf.call({
    route: "create_func",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
    body: createFuncPayload,
  });
  // now list funcs and let's make sure we see it
  const funcs = await sdf.call({
    route: "func_list",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });

  const createdFunc = funcs.find((f) => f.name === name);
  assert(createdFunc, "Expected to find newly created func");
  const funcId = createdFunc.funcId;
  const codePayload = {
    code,
    requestUlid: changeSetId,
  };

  // save the code
  const updateFuncResp = await sdf.call({
    route: "update_func_code",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      funcId,
    },
    body: codePayload,
  });
  return funcId;
}

// MV Helpers -------------------------------------------------------

export async function abandon_all_change_sets(sdf: SdfApiClient) {
  const workspaceId = sdf.workspaceId;
  const data = await sdf.call({
    route: "list_open_change_sets",
    routeVars: {
      workspaceId,
    },
  });
  // loop through them and abandon each one that's not head
  assert(data.defaultChangeSetId, "Expected headChangeSetId");
  const changeSetsToAbandon = data.changeSets.filter(
    (c) => c.id !== data.defaultChangeSetId,
  );

  for (const changeSet of changeSetsToAbandon) {
    const changeSetId = changeSet.id;
    await sdf.call({
      route: "abandon_change_set",
      routeVars: {
        workspaceId: sdf.workspaceId,
        changeSetId,
      },
    });
  }
}

export async function getActions(sdf: SdfApiClient, changeSetId: string) {
  const actions = await sdf.changeSetMjolnir(
    changeSetId,
    "ActionViewList",
    sdf.workspaceId,
  );
  assert(actions, "Expected to get actions MV");
  return actions;
}

// Used to generate the correct payload for sdf
// If not given a Schema Id, assume it's a builtin, and find
// the Id by the name by looking at the CachedSchemas Deployment MV
// If given a schema Id, just try to find an installed one
export async function createComponentPayload(
  sdf: SdfApiClient,
  changeSetId: string,
  schemaName: string,
  schemaId?: string,
) {
  const deploymentIndex = await sdf.fetchDeploymentIndex();
  // console.log(deploymentIndex);

  const cachedSchemasMV = deploymentIndex.frontEndObject.data.mvList.find(
    (mv: any) => mv.kind === "CachedSchemas",
  );
  const cachedSchemasMVId = cachedSchemasMV.id;
  let actualSchemaId = undefined;
  if (!schemaId) {
    // Get the builtin version, so we can get the Schema Id as we only have the name
    const builtins = await sdf.deploymentMjolnir(
      "CachedSchemas",
      cachedSchemasMVId,
    );

    assert(builtins, "Expected to get schemas MV");
    let foundSchema = builtins.schemas.find(
      (schema: { id: string; name: string }) => schema.name === schemaName,
    );
    assert(foundSchema, `Expected to find id for Schema Name ${schemaName}`);
    actualSchemaId = foundSchema.id;
    assert(actualSchemaId, `Expected to find id for Schema Name ${schemaName}`);
  } else {
    actualSchemaId = schemaId;
  }

  assert(actualSchemaId, "we have a valid schema Id!");
  try {
    // First is it installed?
    const maybeInstalled = await sdf.changeSetMjolnir(
      changeSetId,
      "LuminorkDefaultVariant",
      actualSchemaId,
    );
    // if so, return it
    return {
      schemaVariantId: maybeInstalled.variantId,
      x: "0",
      y: "0",
      height: "0",
      width: "0",
      parentId: null,
      schemaType: "installed",
    };
  } catch (e) {
    console.log(
      `LuminorkDefaultVariant not found for ${schemaName}, assuming it's uninstalled`,
    );
    return {
      schemaId: actualSchemaId,
      schemaType: "uninstalled",
      x: "0",
      y: "0",
      height: "0",
      width: "0",
      parentId: null,
    };
  }
}

export async function getViews(sdf: SdfApiClient, changeSetId: string) {
  const viewsList = await sdf.changeSetMjolnir(
    changeSetId,
    "ViewList",
    sdf.workspaceId,
  );
  assert(viewsList, "Expected to get views MV");
  let viewsToFetch = viewsList.views;
  let views = await sdf.changeSetMultiMjolnir(changeSetId, viewsToFetch);
  assert(views, "Expected to get views data");
  return views;
}

export async function eventualMVAssert(
  sdf: SdfApiClient,
  changeSetId: string,
  kind: string,
  id: string,
  assertFn: (mv: any) => boolean,
  message: string,
  timeoutMs: number = 90000, // 90 seconds which should be plenty of time, but we're seeing some timing issues in crons so bumping it up dramatically
): Promise<void> {
  // update this to use sdf.mjolnir and retryUntil
  if (!sdf || !changeSetId || !kind || !id) {
    throw new Error("Invalid parameters for eventualAssert");
  }
  await retryUntil(
    async () => {
      const mv = await sdf.changeSetMjolnir(changeSetId, kind, id);
      if (mv) {
        try {
          if (assertFn(mv)) {
            return; // Success!
          } else {
            throw new Error(`Assertion failed for ${kind} with ID ${id}`);
          }
        } catch (error) {
          // Continue polling if assertion throws
          throw error; // Re-throw to trigger retry
        }
      } else {
        throw new Error(`No MV found for ${kind} with ID ${id}`);
      }
    },
    timeoutMs,
    message,
  );
}
