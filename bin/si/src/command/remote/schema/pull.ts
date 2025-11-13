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
import {
  functionKindToMaterializableEntity,
  MaterializableEntity,
} from "../../../materialize.ts";
import { getLogger } from "../../../logger.ts";
import {
  AbsoluteDirectoryPath,
  AbsoluteFilePath,
  FunctionKind,
  normalizeFsName,
  Project,
} from "../../../project.ts";

const logger = getLogger();

/**
 * Detects if a schema name is a search pattern (e.g., "Fastly::*")
 */
function isSearchPattern(schemaName: string): boolean {
  return schemaName.includes("*");
}

/**
 * Extracts the category from a search pattern like "Fastly::*"
 * Returns the category or null if pattern is invalid
 */
function extractCategoryFromPattern(pattern: string): string | null {
  // Match patterns like "Category::*" or "Category::Subcategory::*"
  const match = pattern.match(/^(.+)::\*$/);
  if (match && match[1]) {
    return match[1];
  }
  // If pattern is just "*", return null to search all
  if (pattern === "*") {
    return null;
  }
  return null;
}

/**
 * Searches for schemas using the search API
 */
async function searchSchemas(
  api: Api,
  changeSetCoord: ChangeSetCoordinate,
  pattern: string,
): Promise<string[]> {
  const category = extractCategoryFromPattern(pattern);

  logger.info(`Searching for schemas matching pattern: ${pattern}`);
  if (category) {
    logger.info(`  Using category filter: ${category}`);
  } else {
    logger.info(`  Searching all schemas`);
  }

  const response = await api.schemas.searchSchemas({
    workspaceId: changeSetCoord.workspaceId,
    changeSetId: changeSetCoord.changeSetId,
    searchSchemasV1Request: {
      category: category,
    },
  });

  const schemaNames = response.data.schemas.map((s) => s.schemaName);
  logger.info(`  Found ${schemaNames.length} matching schema(s)`);

  return schemaNames;
}

export async function callRemoteSchemaPull(
  ctx: Context,
  project: Project,
  apiCtx: ApiContext,
  schemaNames: string[],
  includeBuiltins = false,
) {
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

  // Expand any search patterns into actual schema names
  const expandedSchemaNames: string[] = [];
  for (const schemaName of schemaNames) {
    if (isSearchPattern(schemaName)) {
      const matchingSchemas = await searchSchemas(
        api,
        changeSetCoord,
        schemaName,
      );
      expandedSchemaNames.push(...matchingSchemas);
    } else {
      expandedSchemaNames.push(schemaName);
    }
  }

  if (expandedSchemaNames.length === 0) {
    logger.warn("No schemas found matching the search criteria");
    return [];
  }

  logger.info("");
  logger.info(`Pulling ${expandedSchemaNames.length} schema(s)...`);
  logger.info("");

  const results = [];
  const pulledSchemaNames = [];
  const notFoundSchemaNames = [];
  for (const schemaName of expandedSchemaNames) {
    const result = await pullSchemaByName(
      project,
      api,
      changeSetCoord,
      schemaName,
      includeBuiltins,
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
    isBuiltin: boolean;
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
  overlays: {
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
  includeBuiltins = false,
) {
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

  if (data.variant.isBuiltin) {
    logger.info("{schemaName} is a builtin", { schemaName });

    if (!includeBuiltins) {
      throw new Error(
        `Cannot pull builtin schema "${schemaName}" without --builtins flag. ` +
        `Builtin schemas are schemas you don't own. Use --builtins to pull them.`,
      );
    }

    logger.info(
      "{schemaName}: pulling builtin schema",
      { schemaName },
    );
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
    false,
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

  // Render funcs
  const { paths: actionPaths, deletedPaths: deletedActionPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Action,
      data.func.actionIds,
      false,
    );

  const { paths: authPaths, deletedPaths: deletedAuthPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Auth,
      data.func.authIds,
      false,
    );

  const { paths: codegenPaths, deletedPaths: deletedCodegenPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Codegen,
      data.func.codegenIds,
      false,
    );

  const { paths: managementPaths, deletedPaths: deletedManagementPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Management,
      data.func.managementIds,
      false,
    );

  const { paths: qualificationPaths, deletedPaths: deletedQualificationPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Qualification,
      data.func.qualificationIds,
      false,
    );

  const { paths: overlayActionPaths, deletedPaths: deletedOverlayActionPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Action,
      data.overlays.actionIds,
      true,
    );

  const { paths: overlayAuthPaths, deletedPaths: deletedOverlayAuthPaths } =
    await pullFunctionsByKind(
      project,
      api,
      changeSetCoord,
      schemaName,
      FunctionKind.Auth,
      data.overlays.authIds,
      true,
    );

  const {
    paths: overlayCodegenPaths,
    deletedPaths: deletedOverlayCodegenPaths,
  } = await pullFunctionsByKind(
    project,
    api,
    changeSetCoord,
    schemaName,
    FunctionKind.Codegen,
    data.overlays.codegenIds,
    true,
  );

  const {
    paths: overlayManagementPaths,
    deletedPaths: deletedOverlayManagementPaths,
  } = await pullFunctionsByKind(
    project,
    api,
    changeSetCoord,
    schemaName,
    FunctionKind.Management,
    data.overlays.managementIds,
    true,
  );

  const {
    paths: overlayQualificationPaths,
    deletedPaths: deletedOverlayQualificationPaths,
  } = await pullFunctionsByKind(
    project,
    api,
    changeSetCoord,
    schemaName,
    FunctionKind.Qualification,
    data.overlays.qualificationIds,
    true,
  );

  // TODO(victor) Tidy this up, too many fields
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
    deletedAuthPaths,
    deletedCodegenPaths,
    deletedManagementPaths,
    deletedQualificationPaths,
    overlayActionPaths,
    overlayAuthPaths,
    overlayCodegenPaths,
    overlayManagementPaths,
    overlayQualificationPaths,
    deletedOverlayActionPaths,
    deletedOverlayAuthPaths,
    deletedOverlayCodegenPaths,
    deletedOverlayManagementPaths,
    deletedOverlayQualificationPaths,
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
      isBuiltin: variant.installedFromUpstream,
      displayName: variant.displayName,
      description: variant.description,
      category: variant.category,
      link: variant.link,
    },
    func: {
      schemaId: variant.assetFuncId,
      actionIds: funcs.nonOverlays().byKind(FunctionKind.Action).ids(),
      authIds: funcs.nonOverlays().byKind(FunctionKind.Auth).ids(),
      codegenIds: funcs.nonOverlays().byKind(FunctionKind.Codegen).ids(),
      managementIds: funcs.nonOverlays().byKind(FunctionKind.Management).ids(),
      qualificationIds: funcs.nonOverlays().byKind(FunctionKind.Qualification)
        .ids(),
    },
    overlays: {
      actionIds: funcs.overlays().byKind(FunctionKind.Action).ids(),
      authIds: funcs.overlays().byKind(FunctionKind.Auth).ids(),
      codegenIds: funcs.overlays().byKind(FunctionKind.Codegen).ids(),
      managementIds: funcs.overlays().byKind(FunctionKind.Management).ids(),
      qualificationIds: funcs.overlays().byKind(FunctionKind.Qualification)
        .ids(),
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
) {
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
  funcIds: string[],
  isOverlay: boolean, // TODO(victor) this argument is not the best way to get this. Take in module only instead of project?
): Promise<{
  paths: { metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }[];
  deletedPaths: {
    metadataPath: AbsoluteFilePath;
    codePath: AbsoluteFilePath;
  }[];
}> {
  const paths = [];
  const deletedPaths = [];
  const entity = functionKindToMaterializableEntity(functionKind);

  const module = isOverlay ? project.overlays : project.schemas;
  const basePath = module.funcBasePath(schemaName, functionKind);
  const existingNames = await listFunctionNamesInDir(basePath);

  if (funcIds.length > 0) {
    await materialize.materializeEntityBase(
      project,
      entity,
      schemaName,
      isOverlay,
    );
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
      {
        overwrite: true,
        isOverlay,
      },
    );
    paths.push(result);
    pulledNames.push(normalizeFsName(metadata.name));
  }

  // Delete functions that exist locally but weren't in the remote data
  for (const localName of existingNames) {
    if (!pulledNames.includes(localName)) {
      const metadataPath = module.funcMetadataPath(
        schemaName,
        localName,
        functionKind,
      );
      const codePath = module.funcCodePath(
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

  public byKind(kind: FunctionKind) {
    let filtered: SchemaVariantFunc[];
    switch (kind) {
      case FunctionKind.Action:
        filtered = this.funcs.filter((svf) => svf.funcKind.kind === "action");
        break;
      case FunctionKind.Auth:
        filtered = this.funcs.filter(
          (svf) =>
            svf.funcKind.kind === "other" &&
            svf.funcKind.funcKind === "Authentication",
        );
        break;
      case FunctionKind.Codegen:
        filtered = this.funcs.filter(
          (svf) =>
            svf.funcKind.kind === "other" &&
            svf.funcKind.funcKind === "CodeGeneration",
        );
        break;
      case FunctionKind.Management:
        filtered = this.funcs.filter((svf) =>
          svf.funcKind.kind === "management"
        );
        break;
      case FunctionKind.Qualification:
        filtered = this.funcs.filter(
          (svf) =>
            svf.funcKind.kind === "other" &&
            svf.funcKind.funcKind === "Qualification",
        );
        break;
      default:
        throw new Error(`Unknown function kind: ${kind}`);
    }

    return new SchemaVariantFuncs(filtered);
  }

  public overlays() {
    return new SchemaVariantFuncs(this
      .funcs
      .filter((f) => f.isOverlay));
  }

  public nonOverlays() {
    return new SchemaVariantFuncs(this
      .funcs
      .filter((f) => !f.isOverlay));
  }

  public ids() {
    return this.funcs.map((func) => func.id);
  }
}
