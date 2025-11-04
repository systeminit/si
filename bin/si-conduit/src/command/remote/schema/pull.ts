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
  // First, get list of existing local actions before pulling
  const actionBasePath = project.schemas.funcBasePath(
    schemaName,
    FunctionKind.Action,
  );
  const existingActionNames = await listFunctionNamesInDir(actionBasePath);

  if (data.func.actionIds.length > 0) {
    await materialize.materializeEntityBase(
      project,
      MaterializableEntity.Action,
      schemaName,
    );
  }
  const actionPaths = [];
  const pulledActionNames = [];
  for (const funcId of data.func.actionIds) {
    const { metadata, codeBody } = await fetchFuncMetadataAndCode(api, {
      funcId,
      ...changeSetCoord,
    });
    const metadataBody = JSON.stringify(metadata, null, 2);

    const paths = await materialize.materializeEntity(
      project,
      { entity: MaterializableEntity.Action, schemaName, name: metadata.name },
      metadataBody,
      codeBody,
      { overwrite: true },
    );
    actionPaths.push(paths);
    pulledActionNames.push(normalizeFsName(metadata.name));
  }

  // Delete actions that exist locally but weren't in the remote data
  const deletedActionPaths = [];
  for (const localActionName of existingActionNames) {
    if (!pulledActionNames.includes(localActionName)) {
      const metadataPath = project.schemas.funcMetadataPath(
        schemaName,
        localActionName,
        FunctionKind.Action,
      );
      const codePath = project.schemas.funcCodePath(
        schemaName,
        localActionName,
        FunctionKind.Action,
      );

      await deleteFunctionFiles(project, metadataPath, codePath);
      deletedActionPaths.push({ metadataPath, codePath });
    }
  }

  // Clean up actions directory if it's now empty
  if (existingActionNames.length > 0 && pulledActionNames.length === 0) {
    await deleteIfEmpty(actionBasePath);
  }

  // Render all auth funcs
  // First, get list of existing local authentication functions before pulling
  const authBasePath = project.schemas.funcBasePath(
    schemaName,
    FunctionKind.Auth,
  );
  const existingAuthNames = await listFunctionNamesInDir(authBasePath);

  if (data.func.authIds.length > 0) {
    await materialize.materializeEntityBase(
      project,
      MaterializableEntity.Auth,
      schemaName,
    );
  }
  const authPaths = [];
  const pulledAuthNames = [];
  for (const funcId of data.func.authIds) {
    const { metadata, codeBody } = await fetchFuncMetadataAndCode(api, {
      funcId,
      ...changeSetCoord,
    });
    const metadataBody = JSON.stringify(metadata, null, 2);

    const paths = await materialize.materializeEntity(
      project,
      { entity: MaterializableEntity.Auth, schemaName, name: metadata.name },
      metadataBody,
      codeBody,
      { overwrite: true },
    );
    authPaths.push(paths);
    pulledAuthNames.push(normalizeFsName(metadata.name));
  }

  // Delete authentication functions that exist locally but weren't in the
  // remote data
  const deletedAuthPaths = [];
  for (const localAuthName of existingAuthNames) {
    if (!pulledAuthNames.includes(localAuthName)) {
      const metadataPath = project.schemas.funcMetadataPath(
        schemaName,
        localAuthName,
        FunctionKind.Auth,
      );
      const codePath = project.schemas.funcCodePath(
        schemaName,
        localAuthName,
        FunctionKind.Auth,
      );

      await deleteFunctionFiles(project, metadataPath, codePath);
      deletedAuthPaths.push({ metadataPath, codePath });
    }
  }

  // Clean up authentication directory if it's now empty
  if (existingAuthNames.length > 0 && pulledAuthNames.length === 0) {
    await deleteIfEmpty(authBasePath);
  }

  // Render all codegen funcs
  // First, get list of existing local codegens before pulling
  const codegenBasePath = project.schemas.funcBasePath(
    schemaName,
    FunctionKind.Codegen,
  );
  const existingCodegenNames = await listFunctionNamesInDir(codegenBasePath);

  if (data.func.codegenIds.length > 0) {
    await materialize.materializeEntityBase(
      project,
      MaterializableEntity.Codegen,
      schemaName,
    );
  }
  const codegenPaths = [];
  const pulledCodegenNames = [];
  for (const funcId of data.func.codegenIds) {
    const { metadata, codeBody } = await fetchFuncMetadataAndCode(api, {
      funcId,
      ...changeSetCoord,
    });
    const metadataBody = JSON.stringify(metadata, null, 2);

    const paths = await materialize.materializeEntity(
      project,
      { entity: MaterializableEntity.Codegen, schemaName, name: metadata.name },
      metadataBody,
      codeBody,
      { overwrite: true },
    );
    codegenPaths.push(paths);
    pulledCodegenNames.push(normalizeFsName(metadata.name));
  }

  // Delete codegens that exist locally but weren't in the remote data
  const deletedCodegenPaths = [];
  for (const localCodegenName of existingCodegenNames) {
    if (!pulledCodegenNames.includes(localCodegenName)) {
      const metadataPath = project.schemas.funcMetadataPath(
        schemaName,
        localCodegenName,
        FunctionKind.Codegen,
      );
      const codePath = project.schemas.funcCodePath(
        schemaName,
        localCodegenName,
        FunctionKind.Codegen,
      );

      await deleteFunctionFiles(project, metadataPath, codePath);
      deletedCodegenPaths.push({ metadataPath, codePath });
    }
  }

  // Clean up codegens directory if it's now empty
  if (existingCodegenNames.length > 0 && pulledCodegenNames.length === 0) {
    await deleteIfEmpty(codegenBasePath);
  }

  // Render all management funcs
  // First, get list of existing local management functions before pulling
  const managementBasePath = project.schemas.funcBasePath(
    schemaName,
    FunctionKind.Management,
  );
  const existingManagementNames = await listFunctionNamesInDir(
    managementBasePath,
  );

  if (data.func.managementIds.length > 0) {
    await materialize.materializeEntityBase(
      project,
      MaterializableEntity.Management,
      schemaName,
    );
  }
  const managementPaths = [];
  const pulledManagementNames = [];
  for (const funcId of data.func.managementIds) {
    const { metadata, codeBody } = await fetchFuncMetadataAndCode(api, {
      funcId,
      ...changeSetCoord,
    });
    const metadataBody = JSON.stringify(metadata, null, 2);

    const paths = await materialize.materializeEntity(
      project,
      {
        entity: MaterializableEntity.Management,
        schemaName,
        name: metadata.name,
      },
      metadataBody,
      codeBody,
      { overwrite: true },
    );
    managementPaths.push(paths);
    pulledManagementNames.push(normalizeFsName(metadata.name));
  }

  // Delete management functions that exist locally but weren't in the remote data
  const deletedManagementPaths = [];
  for (const localManagementName of existingManagementNames) {
    if (!pulledManagementNames.includes(localManagementName)) {
      const metadataPath = project.schemas.funcMetadataPath(
        schemaName,
        localManagementName,
        FunctionKind.Management,
      );
      const codePath = project.schemas.funcCodePath(
        schemaName,
        localManagementName,
        FunctionKind.Management,
      );

      await deleteFunctionFiles(project, metadataPath, codePath);
      deletedManagementPaths.push({ metadataPath, codePath });
    }
  }

  // Clean up management directory if it's now empty
  if (
    existingManagementNames.length > 0 &&
    pulledManagementNames.length === 0
  ) {
    await deleteIfEmpty(managementBasePath);
  }

  // Render all qualification funcs
  // First, get list of existing local qualifications before pulling
  const qualificationBasePath = project.schemas.funcBasePath(
    schemaName,
    FunctionKind.Qualification,
  );
  const existingQualificationNames = await listFunctionNamesInDir(
    qualificationBasePath,
  );

  if (data.func.qualificationIds.length > 0) {
    await materialize.materializeEntityBase(
      project,
      MaterializableEntity.Qualification,
      schemaName,
    );
  }
  const qualificationPaths = [];
  const pulledQualificationNames = [];
  for (const funcId of data.func.qualificationIds) {
    const { metadata, codeBody } = await fetchFuncMetadataAndCode(api, {
      funcId,
      ...changeSetCoord,
    });
    const metadataBody = JSON.stringify(metadata, null, 2);

    const paths = await materialize.materializeEntity(
      project,
      {
        entity: MaterializableEntity.Qualification,
        schemaName,
        name: metadata.name,
      },
      metadataBody,
      codeBody,
      { overwrite: true },
    );
    qualificationPaths.push(paths);
    pulledQualificationNames.push(normalizeFsName(metadata.name));
  }

  // Delete qualifications that exist locally but weren't in the remote data
  const deletedQualificationPaths = [];
  for (const localQualificationName of existingQualificationNames) {
    if (!pulledQualificationNames.includes(localQualificationName)) {
      const metadataPath = project.schemas.funcMetadataPath(
        schemaName,
        localQualificationName,
        FunctionKind.Qualification,
      );
      const codePath = project.schemas.funcCodePath(
        schemaName,
        localQualificationName,
        FunctionKind.Qualification,
      );

      await deleteFunctionFiles(project, metadataPath, codePath);
      deletedQualificationPaths.push({ metadataPath, codePath });
    }
  }

  // Clean up qualifications directory if it's now empty
  if (
    existingQualificationNames.length > 0 &&
    pulledQualificationNames.length === 0
  ) {
    await deleteIfEmpty(qualificationBasePath);
  }

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
