import { SdfApiClient } from "./sdf_api_client.ts";
import assert from "node:assert";
import { ulid } from "https://deno.land/x/ulid@v0.3.0/mod.ts";


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
  retries = 20,
  backoffFactor = 1.2, // exponential backoff factor
  initialDelay = .4, // in seconds
): Promise<void> {
  const retryPromise = retryWithBackoff(fn, retries, backoffFactor, initialDelay);
  
  const timeoutPromise = new Promise<void>((_, reject) => {
    setTimeout(() => {
      reject(new Error(`Timeout after ${timeoutMs}ms while retrying operation`));
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
      changeSetName,
    },
  });
  assert(typeof data.changeSet === "object", "Expected changeSet in response");
  const changeSet = data.changeSet;
  assert(changeSet?.id, "Expected Change Set id");
  assert(
    changeSet?.name === changeSetName,
    `Changeset name should be ${changeSetName}`,
  );
  const changeSetId = changeSet.id;

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
    body: {
      changeSetId,
    },
  });

  if (err) {
    throw err;
  }
}

export const nilId = "00000000000000000000000000";

// Diagram Helpers ------------------------------------------------------------


export async function getQualificationSummary(sdf: SdfApiClient, changeSetId: string) {
  return await sdf.call({
    route: "qualification_summary",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
}

export async function getActions(sdf: SdfApiClient, changeSetId: string) {
  return await sdf.call({
    route: "action_list",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
}

export async function getFuncs(sdf: SdfApiClient, changeSetId: string) {
  return await sdf.call({
    route: "func_list",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
    },
  });
}


// Prop Helpers ------------------------------------------------------------


export async function getPropertyEditor(
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

export async function installModule(
  sdf: SdfApiClient,
  changeSetId: string,
  moduleId: string,
) {
  const installModulePayload = {
    visibility_change_set_pk: changeSetId,
    ids: [moduleId]
  };

  await sdf.call({
    route: "install_module",
    body: installModulePayload,
  });
}
export function extractSchemaVariant(
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

export async function getSchemaVariants(sdf: SdfApiClient, changeSetId: string) {
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

export async function setAttributeValue(
  sdf: SdfApiClient,
  changeSetId: string,
  componentId: string,
  attributeValueId: string,
  parentAttributeValueId: string,
  propId: string,
  value: unknown,
) {
  const updateValuePayload = {
    visibility_change_set_pk: changeSetId,
    componentId,
    attributeValueId,
    parentAttributeValueId,
    propId,
    value,
    isForSecret: false,
  };

  await sdf.call({
    route: "update_property_value",
    body: updateValuePayload,
  });
}

export function attributeValueIdForPropPath(
  propPath: string,
  propList: any[],
  attributeValuesView: {
    values: any[];
    childValues: any[];
  },
) {
  const prop = propList.find((p) => p.path === propPath);
  assert(prop, `Expected to find ${propPath} prop`);

  let attributeValueId;
  let value;
  for (const attributeValue in attributeValuesView.values) {
    if (attributeValuesView.values[attributeValue]?.propId === prop.id) {
      attributeValueId = attributeValue;
      value = attributeValuesView.values[attributeValue]?.value;
    }
  }
  assert(attributeValueId, "Expected source attribute value");

  let parentAttributeValueId;
  for (const attributeValue in attributeValuesView?.childValues) {
    const avChildren = attributeValuesView?.childValues[attributeValue] ?? [];
    if (avChildren.includes(attributeValueId)) {
      parentAttributeValueId = attributeValue;
    }
  }
  assert(parentAttributeValueId, "Expected parent of source attribute value");

  return {
    attributeValueId,
    parentAttributeValueId,
    propId: prop.id,
    value,
  };
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

  const createResp = await sdf.call({ route: "create_variant", body: createAssetPayload });
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
    route: "get_variant", routeVars: {
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

// Func Helpers ------------------------------------------------------------

export async function getFuncRun(sdf: SdfApiClient, changeSetId: string, funcRunId: string) {
  console.log(funcRunId);
  const funcRun = await sdf.call({
    route: "get_func_run",
    routeVars: {
      workspaceId: sdf.workspaceId,
      changeSetId,
      funcRunId,
    }
  });
  return funcRun;
}
export async function testExecuteFunc(sdf: SdfApiClient, changeSetId: string, funcId: string, componentId: string, code: string, args: any) {
  const testExecuteFuncPayload = {
    args,
    code,
    componentId
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

export async function createQualification(sdf: SdfApiClient, changeSetId: string, name: string, schemaVariantId: string, code: string) {
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

export function createComponentPayload(schemaVariantCategoriesMV: any, schemaName: string) {
  const installedVariant = schemaVariantCategoriesMV.installed?.find((sv: any) => sv.schemaName === schemaName);
  if (installedVariant) {
    return {
      schemaVariantId: installedVariant.schemaVariantId,
      x: "0",
      y: "0",
      height: "0",
      width: "0",
      parentId: null,
      schemaType: "installed",
    };
  }
  else {
    const uninstalledVariant = schemaVariantCategoriesMV.uninstalled?.find((sv: any) => sv.schemaName === schemaName);
    assert(uninstalledVariant, `Expected to find ${schemaName} schema variant`);
    return {
      schemaId: uninstalledVariant.schemaId,
      schemaType: "uninstalled",
      x: "0",
      y: "0",
      height: "0",
      width: "0",
      parentId: null,
    }
  }
}


// MV Assertion Helpers -------------------------------------------------------


export async function getVariants(sdf: SdfApiClient, changeSetId: string) {
  const schemas = await sdf.mjolnir(changeSetId, "SchemaVariantCategories", sdf.workspaceId);
  assert(schemas, "Expected to get schemas MV");

  const installedList:{kind:string, id:string}[] = schemas.categories.flatMap((c: any) => c.schemaVariants.filter((v: any) => v.type === "installed").map((v: any) => {
    return {
      kind: "SchemaVariant",
      id: v.id,
    }
  }));
  let installed = [];
  if (installedList.length > 0) {
    let installedResp = await sdf.multiMjolnir(changeSetId, installedList);
    assert(installedResp, "Expected to get installed variants data");
    installed = installedResp;
  }

  const uninstalled = schemas.categories.flatMap((c: any) =>
    c.schemaVariants
      .filter((v: any) => v.type === "uninstalled")
      .map((v: any) => {
        const uninstalledMeta = schemas.uninstalled?.[v.id];
        assert(uninstalledMeta, `Expected uninstalled metadata for schemaId ${v.id}`);
        return uninstalledMeta;
      })
  );
  return { installed, uninstalled };
}


export async function getViews(sdf: SdfApiClient, changeSetId: string) {
  const viewsList = await sdf.mjolnir(changeSetId, "ViewList", sdf.workspaceId);
  assert(viewsList, "Expected to get views MV");
  let viewsToFetch = viewsList.views;
  let views = await sdf.multiMjolnir(changeSetId, viewsToFetch);
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
   timeoutMs: number = 15000, // 15 seconds

): Promise<void> {
  // update this to use sdf.mjolnir and retryUntil 
  if (!sdf || !changeSetId || !kind || !id) {
    throw new Error("Invalid parameters for eventualAssert");
  } 
  try {
    await retryUntil(async () => {
      const mv = await sdf.mjolnir(changeSetId, kind, id);
      if (mv) {
        try {
          if (assertFn(mv)) {
            return; // Success!
          }
        } catch (error) {
          // Continue polling if assertion throws
          console.warn(`Assertion failed for ${kind} with ID ${id}:`, error);
        }
      }
    }, timeoutMs);
  } catch (err) {
    throw new Error(`Timeout after ${timeoutMs}ms: ${message}`);
  }
}

