import {
  FuncsApi,
  SchemasApi,
  SchemaVariantFunc,
} from "@systeminit/api-client";
import { ApiContext } from "../../../api.ts";
import { SCHEMA_FILE_FORMAT_VERSION } from "../../../config.ts";
import { Context } from "../../../context.ts";
import { FunctionMetadata, SchemaMetadata } from "../../../generators.ts";
import * as materialize from "../../../materialize.ts";
import { MaterializableEntity } from "../../../materialize.ts";
import { getLogger } from "../../../logger.ts";
import {
  AbsoluteDirectoryPath,
  AbsoluteFilePath,
  FunctionKind,
  normalizeFsName,
  Project,
} from "../../../project.ts";

const logger = getLogger();

export async function callRemoteSchemaPull(
  ctx: Context,
  project: Project,
  apiCtx: ApiContext,
  schemaNames: string[],
): Promise<PullFullSchemaResult[]> {
  logger.info("Pulling remote schemas to local system");
  logger.info("---");
  logger.info("");

  const api = {
    schemas: new SchemasApi(apiCtx.config),
    funcs: new FuncsApi(apiCtx.config),
  };

  const changeSetCoord = {
    workspaceId: apiCtx.workspace.id,
    changeSetId: "HEAD",
  };

  const results = [];
  const pulledSchemaNames = [];
  const notFoundSchemaNames = [];
  for (const schemaName of schemaNames) {
    const result = await pullSchemaByName(
      project,
      api,
      changeSetCoord,
      schemaName,
    );
    if (result) {
      results.push(result);
      pulledSchemaNames.push(schemaName);
    } else {
      notFoundSchemaNames.push(schemaName);
    }
  }

  ctx.analytics.trackEvent("remote_schema_pull", {
    schemaNames: pulledSchemaNames,
    notFoundSchemaNames,
  });

  // TODO(fnichol): If some schemas were not found, should we error out? At the
  // moment we aren't

  logger.info("");
  logger.info("---");
  logger.info("Successfully pulled schemas:");
  for (const schemaName of pulledSchemaNames) {
    logger.info(` - ${schemaName}`);
  }

  return results;
}

type Api = {
  schemas: SchemasApi;
  funcs: FuncsApi;
};

type SchemaAndVariantData = {
  schema: {
    id: string;
    name: string;
  };
  variant: {
    id: string;
    displayName: string;
    description?: string | null;
    category: string;
    link?: string | null;
  };
  func: {
    schemaId: string;
    actionIds: string[];
    authIds: string[];
    codegenIds: string[];
    managementIds: string[];
    qualificationIds: string[];
  };
};

type ChangeSetCoordinate = {
  workspaceId: string;
  changeSetId: string;
};

type FuncCoord = ChangeSetCoordinate & {
  funcId: string;
};

async function pullSchemaByName(
  project: Project,
  api: Api,
  changeSetCoord: ChangeSetCoordinate,
  schemaName: string,
): Promise<PullFullSchemaResult | undefined> {
  logger.info("Pulling schema {schemaName}", { schemaName });

  const data = await getSchemaAndVariantBySchemaName(
    api,
    changeSetCoord,
    schemaName,
  );

  if (!data) {
    // TODO(fnichol): log error, throw, or other here?
    return undefined;
  }

  const formatVersionBody = SCHEMA_FILE_FORMAT_VERSION.toString();

  const metadata = schemaMetadata(data);
  const metadataBody = JSON.stringify(metadata, null, 2);

  const { codeBody } = await fetchFuncMetadataAndCode(api, {
    funcId: data.func.schemaId,
    ...changeSetCoord,
  });

  await materialize.materializeEntityBase(
    project,
    MaterializableEntity.Schema,
    schemaName,
  );
  const { formatVersionPath } = await materialize
    .materializeSchemaFormatVersion(
      project,
      schemaName,
      formatVersionBody,
      { overwrite: true },
    );
  const { metadataPath, codePath } = await materialize.materializeEntity(
    project,
    { entity: MaterializableEntity.Schema, name: schemaName },
    metadataBody,
    codeBody,
    { overwrite: true },
  );

  // Render all action funcs
  const { paths: actionPaths, deletedPaths: deletedActionPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Action,
      MaterializableEntity.Action,
      data.func.actionIds,
    );

  // Render all auth funcs
  const { paths: authPaths, deletedPaths: deletedAuthPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Auth,
      MaterializableEntity.Auth,
      data.func.authIds,
    );

  // Render all codegen funcs
  const { paths: codegenPaths, deletedPaths: deletedCodegenPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Codegen,
      MaterializableEntity.Codegen,
      data.func.codegenIds,
    );

  // Render all management funcs
  const { paths: managementPaths, deletedPaths: deletedManagementPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Management,
      MaterializableEntity.Management,
      data.func.managementIds,
    );

  // Render all qualification funcs
  const { paths: qualificationPaths, deletedPaths: deletedQualificationPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Qualification,
      MaterializableEntity.Qualification,
      data.func.qualificationIds,
    );

  return {
    formatVersionPath,
    metadataPath,
    codePath,
    actionPaths,
    authPaths,
    codegenPaths,
    managementPaths,
    qualificationPaths,
    deletedActionPaths,
    deletedCodegenPaths,
    deletedManagementPaths,
    deletedQualificationPaths,
  };
}

async function fetchFuncMetadataAndCode(
  api: Api,
  funcCoord: FuncCoord,
): Promise<{ metadata: FunctionMetadata; codeBody: string }> {
  const { data: func } = await api.funcs.getFunc({ ...funcCoord });

  return {
    metadata: {
      name: func.name,
      displayName: func.displayName,
      description: func.description,
    },
    codeBody: func.code,
  };
}

async function getSchemaAndVariantBySchemaName(
  api: Api,
  changeSetCoord: ChangeSetCoordinate,
  schemaName: string,
): Promise<SchemaAndVariantData | undefined> {
  const response = await api.schemas.findSchema(
    {
      schema: schemaName,
      ...changeSetCoord,
    },
    {
      // Not found is an expected response
      validateStatus: (status) =>
        (status >= 200 && status < 300) || status == 404,
    },
  );
  const { status: schemaStatus, data: schema } = response;

  if (schemaStatus == 404) {
    // TODO(fnichol): log and return or throw?
    logger.error("remote schema not found named {schemaName}", {
      schemaName,
    });
    return undefined;
  }

  const schemaCoord = {
    schemaId: schema.schemaId,
    ...changeSetCoord,
  };

  // FIXME Here in variant, we'll get an `installedFromUpstream` field that tells us if this variant is a builtin
  const { data: variant } = await api.schemas.getDefaultVariant({
    ...schemaCoord,
  });
  // TODO(fnichol): handle HTTP/202 to try again as variant could be building

  const funcs = new SchemaVariantFuncs(variant.variantFuncs);

  return {
    schema: {
      id: schema.schemaId,
      name: schema.schemaName,
    },
    variant: {
      id: variant.variantId,
      displayName: variant.displayName,
      description: variant.description,
      category: variant.category,
      link: variant.link,
    },
    func: {
      schemaId: variant.assetFuncId,
      actionIds: funcs.actionIds(),
      authIds: funcs.authIds(),
      codegenIds: funcs.codegenIds(),
      managementIds: funcs.managementIds(),
      qualificationIds: funcs.qualificationIds(),
    },
  };
}

/**
 * Lists function names in a directory by finding all .ts files and extracting
 * their base names.
 *
 * Returns an empty array if the directory doesn't exist.
 */
async function listFunctionNamesInDir(
  dirPath: AbsoluteDirectoryPath,
): Promise<string[]> {
  try {
    const entries = [];
    for await (const dirEntry of Deno.readDir(dirPath.path)) {
      if (dirEntry.isFile && dirEntry.name.endsWith(".ts")) {
        // Extract base name without .ts extension
        const baseName = dirEntry.name.slice(0, -3);
        entries.push(baseName);
      }
    }
    return entries;
  } catch (err) {
    if (err instanceof Deno.errors.NotFound) {
      return [];
    }
    throw err;
  }
}

/**
 * Deletes both the metadata and code files for a function.
 */
async function deleteFunctionFiles(
  project: Project,
  metadataPath: AbsoluteFilePath,
  codePath: AbsoluteFilePath,
): Promise<void> {
  if (await metadataPath.exists()) {
    await Deno.remove(metadataPath.path);
    logger.info("  - Deleted: {path}", {
      path: metadataPath.relativeToStr(project),
    });
  }
  if (await codePath.exists()) {
    await Deno.remove(codePath.path);
    logger.info("  - Deleted: {path}", {
      path: codePath.relativeToStr(project),
    });
  }
}

/**
 * Removes a directory if it exists and is empty.
 */
async function deleteIfEmpty(dirPath: AbsoluteDirectoryPath): Promise<void> {
  try {
    // Try to remove the directory - this will only succeed if it's empty
    await Deno.remove(dirPath.path);
  } catch (err) {
    // Ignore errors (directory not empty, doesn't exist, etc.)
    // NotFound means directory doesn't exist (fine)
    // Other errors (like directory not empty) are also fine - we just skip
    // removal
    if (!(err instanceof Deno.errors.NotFound)) {
      // Directory exists but couldn't be removed (likely not empty), which is
      // expected
      logger.trace("Directory {dirPath} not removed (likely not empty)", {
        dirPath: dirPath.path,
      });
    }
  }
}

/**
 * Pulls functions of a specific kind (Action, Auth, Codegen, Management, or Qualification)
 * from the remote API and materializes them locally.
 *
 * Also handles deletion of functions that exist locally but weren't in the remote data.
 */
async function pullFunctionsByKind(
  project: Project,
  api: Api,
  changeSetCoord: ChangeSetCoordinate,
  schemaName: string,
  functionKind: FunctionKind,
  entity: MaterializableEntity,
  funcIds: string[],
): Promise<{
  paths: { metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }[];
  deletedPaths: { metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }[];
}> {
  const paths = [];
  const deletedPaths = [];

  const basePath = project.schemas.funcBasePath(schemaName, functionKind);
  const existingNames = await listFunctionNamesInDir(basePath);

  if (funcIds.length > 0) {
    await materialize.materializeEntityBase(project, entity, schemaName);
  }

  const pulledNames = [];
  for (const funcId of funcIds) {
    const { metadata, codeBody } = await fetchFuncMetadataAndCode(api, {
      funcId,
      ...changeSetCoord,
    });
    const metadataBody = JSON.stringify(metadata, null, 2);

    const result = await materialize.materializeEntity(
      project,
      { entity, schemaName, name: metadata.name },
      metadataBody,
      codeBody,
      { overwrite: true },
    );
    paths.push(result);
    pulledNames.push(normalizeFsName(metadata.name));
  }

  // Delete functions that exist locally but weren't in the remote data
  for (const localName of existingNames) {
    if (!pulledNames.includes(localName)) {
      const metadataPath = project.schemas.funcMetadataPath(
        schemaName,
        localName,
        functionKind,
      );
      const codePath = project.schemas.funcCodePath(
        schemaName,
        localName,
        functionKind,
      );

      await deleteFunctionFiles(project, metadataPath, codePath);
      deletedPaths.push({ metadataPath, codePath });
    }
  }

  // Clean up directory if it's now empty
  if (existingNames.length > 0 && pulledNames.length === 0) {
    await deleteIfEmpty(basePath);
  }

  return { paths, deletedPaths };
}

function schemaMetadata(data: SchemaAndVariantData): SchemaMetadata {
  return {
    name: data.schema.name,
    category: data.variant.category,
    description: data.variant.description,
    documentation: data.variant.link,
  };
}

class SchemaVariantFuncs {
  constructor(public readonly funcs: SchemaVariantFunc[]) {}

  public actionFuncs(): SchemaVariantFunc[] {
    return this.funcs.filter((svf) => svf.funcKind.kind === "action");
  }

  public actionIds(): string[] {
    return this.actionFuncs().map((func) => func.id);
  }

  /**
   * Returns all authentication functions for the schema variant.
   */
  public authFuncs(): SchemaVariantFunc[] {
    return this.funcs.filter(
      (svf) =>
        svf.funcKind.kind === "other" &&
        svf.funcKind.funcKind === "Authentication",
    );
  }

  /**
   * Returns the IDs of all authentication functions.
   */
  public authIds(): string[] {
    return this.authFuncs().map((func) => func.id);
  }

  public codegenFuncs(): SchemaVariantFunc[] {
    return this.funcs.filter(
      (svf) =>
        svf.funcKind.kind === "other" &&
        svf.funcKind.funcKind === "CodeGeneration",
    );
  }

  public codegenIds(): string[] {
    return this.codegenFuncs().map((func) => func.id);
  }

  public managementFuncs(): SchemaVariantFunc[] {
    return this.funcs.filter((svf) => svf.funcKind.kind === "management");
  }

  public managementIds(): string[] {
    return this.managementFuncs().map((func) => func.id);
  }

  public qualificationFuncs(): SchemaVariantFunc[] {
    return this.funcs.filter(
      (svf) =>
        svf.funcKind.kind === "other" &&
        svf.funcKind.funcKind === "Qualification",
    );
  }

  public qualificationIds(): string[] {
    return this.qualificationFuncs().map((func) => func.id);
  }
}

/**
 * Result of pulling a complete schema including all functions and their paths.
 */
export interface PullFullSchemaResult {
  formatVersionPath: AbsoluteFilePath;
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
  actionPaths: { metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }[];
  authPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
  codegenPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
  managementPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
  qualificationPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
  deletedActionPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
  deletedCodegenPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
  deletedManagementPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
  deletedQualificationPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
}
