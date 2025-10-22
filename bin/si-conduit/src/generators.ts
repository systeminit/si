/**
 * Generators Module - Schema and Function Code Generation
 *
 * This module provides code generation utilities for creating schema
 * structures, action functions, code generators, management functions, and
 * qualifications. It handles directory creation, file generation, and template
 * expansion for SI Conduit projects.
 *
 * @example
 * ```ts
 * import { generateSchemaBase, generateSchemaAction } from "./generators.ts";
 * import { Context } from "./context.ts";
 * import { Project } from "./project.ts";
 *
 * const ctx = Context.instance();
 * const project = new Project("/path/to/project");
 *
 * // Generate schema base structure
 * await generateSchemaBase(ctx, project, "MySchema");
 *
 * // Generate an action function
 * await generateSchemaAction(ctx, project, "MySchema", "create");
 * ```
 *
 * @module
 */

import type { Logger } from "@logtape/logtape";
import { SCHEMA_FILE_FORMAT_VERSION } from "./config.ts";
import { Context } from "./context.ts";
import { Project } from "./project.ts";
import type { AbsoluteFilePath } from "./project.ts";

/**
 * Generates the base directory structure for a new schema.
 *
 * Creates the schema directory and a format version file to track
 * compatibility. Throws an error if the schema directory already exists.
 *
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema to create
 * @returns Object containing the path to the created format version file
 * @throws {DirectoryExistsError} If the schema directory already exists
 *
 * @example
 * ```ts
 * const { formatVersionPath } = await generateSchemaBase(ctx, project, "MySchema");
 * ```
 */
export async function generateSchemaBase(
  ctx: Context,
  project: Project,
  schemaName: string,
): Promise<{ formatVersionPath: AbsoluteFilePath }> {
  const logger = ctx.logger;

  const schemaBasePath = project.schemaBasePath(schemaName);

  // Check that the schema directory doesn't exist
  if (await schemaBasePath.exists()) {
    logger.error("Directory already exists at {schemaBasePath}", {
      schemaBasePath: schemaBasePath.toString(),
    });
    throw new DirectoryExistsError(schemaBasePath.toString());
  }

  // Create schema base directory
  await schemaBasePath.mkdir({ recursive: true });
  logger.info("Created schema directory: {schemaBasePath}", {
    schemaBasePath: schemaBasePath.toString(),
  });

  // Create the format version file
  const formatVersionFilePath = project.schemaFormatVersionPath(schemaName);
  await ensureFileDoesNotExist(formatVersionFilePath, logger);
  await createFileWithLogging(
    formatVersionFilePath,
    SCHEMA_FILE_FORMAT_VERSION.toString(),
    logger,
  );

  return {
    formatVersionPath: formatVersionFilePath,
  };
}

/**
 * Generates the actions directory for a schema.
 *
 * Creates the actions directory if it doesn't already exist. This directory
 * will contain action function files (TypeScript code and metadata).
 *
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 */
export async function generateSchemaActionBase(
  ctx: Context,
  project: Project,
  schemaName: string,
) {
  const logger = ctx.logger;

  const actionBasePath = project.actionBasePath(schemaName);

  // Create the action base directory
  if (!(await actionBasePath.exists())) {
    await actionBasePath.mkdir({ recursive: true });
    logger.info("Created actions directory: {actionBasePath}", {
      actionBasePath: actionBasePath.toString(),
    });
  }
}

/**
 * Generates the code generators directory for a schema.
 *
 * Creates the codeGenerators directory if it doesn't already exist. This
 * directory will contain code generator function files.
 *
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 */
export async function generateSchemaCodegenBase(
  ctx: Context,
  project: Project,
  schemaName: string,
) {
  const logger = ctx.logger;

  const codegenBasePath = project.codegenBasePath(schemaName);

  // Create the codegen base directory
  if (!(await codegenBasePath.exists())) {
    await codegenBasePath.mkdir({ recursive: true });
    logger.info("Created code generators directory: {codegenBasePath}", {
      codegenBasePath: codegenBasePath.toString(),
    });
  }
}

/**
 * Generates the management functions directory for a schema.
 *
 * Creates the management directory if it doesn't already exist. This directory
 * will contain management function files.
 *
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 */
export async function generateSchemaManagementBase(
  ctx: Context,
  project: Project,
  schemaName: string,
) {
  const logger = ctx.logger;

  const managementBasePath = project.managementBasePath(schemaName);

  // Create the management base directory
  if (!(await managementBasePath.exists())) {
    await managementBasePath.mkdir({ recursive: true });
    logger.info("Created management directory: {managementBasePath}", {
      managementBasePath: managementBasePath.toString(),
    });
  }
}

/**
 * Generates the qualifications directory for a schema.
 *
 * Creates the qualifications directory if it doesn't already exist. This
 * directory will contain qualification function files.
 *
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 */
export async function generateSchemaQualificationBase(
  ctx: Context,
  project: Project,
  schemaName: string,
) {
  const logger = ctx.logger;

  const qualificationBasePath = project.qualificationBasePath(schemaName);

  // Create the qualification base directory
  if (!(await qualificationBasePath.exists())) {
    await qualificationBasePath.mkdir({ recursive: true });
    logger.info("Created qualifications directory: {qualificationBasePath}", {
      qualificationBasePath: qualificationBasePath.toString(),
    });
  }
}

/**
 * Generates schema metadata and code files.
 *
 * Creates the schema.metadata.json and schema.ts files for a schema. The
 * metadata file contains schema information like name, category, and
 * description. The code file contains the schema definition template.
 *
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 * @returns Object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists
 */
export async function generateSchema(
  ctx: Context,
  project: Project,
  schemaName: string,
): Promise<{ metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }> {
  const logger = ctx.logger;

  // Create schema metadata file
  const metadataFilePath = project.schemaMetadataPath(schemaName);
  await ensureFileDoesNotExist(metadataFilePath, logger);
  const metadata = schemaMetadata(schemaName);
  await createFileWithLogging(
    metadataFilePath,
    JSON.stringify(metadata, null, 2),
    logger,
  );

  // Create schema func code file
  const schemaCodeFilePath = project.schemaFuncCodePath(schemaName);
  await ensureFileDoesNotExist(schemaCodeFilePath, logger);
  await createFileWithLogging(
    schemaCodeFilePath,
    schemaCodeTemplate(schemaName),
    logger,
  );

  return {
    metadataPath: metadataFilePath,
    codePath: schemaCodeFilePath,
  };
}

/**
 * Generates an action function for a schema.
 *
 * Creates both the TypeScript code file and metadata JSON file for an action
 * function. Action functions define operations like create, update, destroy, or
 * refresh for schema instances.
 *
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 * @param actionName - Name of the action function
 * @returns Object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists
 */
export async function generateSchemaAction(
  ctx: Context,
  project: Project,
  schemaName: string,
  actionName: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  const logger = ctx.logger;

  // Create action func metadata file
  const actionMetadataFilePath = project.actionFuncMetadataPath(
    schemaName,
    actionName,
  );
  await ensureFileDoesNotExist(actionMetadataFilePath, logger);
  const metadata = actionMetadata(schemaName, actionName);
  await createFileWithLogging(
    actionMetadataFilePath,
    JSON.stringify(metadata, null, 2),
    logger,
  );

  // Create action func code file
  const actionCodeFilePath = project.actionFuncCodePath(schemaName, actionName);
  await ensureFileDoesNotExist(actionCodeFilePath, logger);
  await createFileWithLogging(
    actionCodeFilePath,
    actionCodeTemplate(schemaName, actionName),
    logger,
  );

  return {
    metadataPath: actionMetadataFilePath,
    codePath: actionCodeFilePath,
  };
}

/**
 * Generates a code generator function for a schema.
 *
 * Creates both the TypeScript code file and metadata JSON file for a code
 * generator function. Code generators produce output code based on the schema's
 * component data.
 *
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 * @param codegenName - Name of the code generator function
 * @returns Object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists
 */
export async function generateSchemaCodegen(
  ctx: Context,
  project: Project,
  schemaName: string,
  codegenName: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  const logger = ctx.logger;

  // Create codegen func metadata file
  const codegenMetadataFilePath = project.codegenFuncMetadataPath(
    schemaName,
    codegenName,
  );
  await ensureFileDoesNotExist(codegenMetadataFilePath, logger);
  const metadata = codegenMetadata(schemaName, codegenName);
  await createFileWithLogging(
    codegenMetadataFilePath,
    JSON.stringify(metadata, null, 2),
    logger,
  );

  // Create codegen func code file
  const codegenCodeFilePath = project.codegenFuncCodePath(
    schemaName,
    codegenName,
  );
  await ensureFileDoesNotExist(codegenCodeFilePath, logger);
  await createFileWithLogging(
    codegenCodeFilePath,
    codegenCodeTemplate(schemaName, codegenName),
    logger,
  );

  return {
    metadataPath: codegenMetadataFilePath,
    codePath: codegenCodeFilePath,
  };
}

/**
 * Generates a management function for a schema.
 *
 * Creates both the TypeScript code file and metadata JSON file for a management
 * function. Management functions handle resource management operations and
 * updates.
 *
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 * @param managementName - Name of the management function
 * @returns Object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists
 */
export async function generateSchemaManagement(
  ctx: Context,
  project: Project,
  schemaName: string,
  managementName: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  const logger = ctx.logger;

  // Create management func metadata file
  const managementMetadataFilePath = project.managementFuncMetadataPath(
    schemaName,
    managementName,
  );
  await ensureFileDoesNotExist(managementMetadataFilePath, logger);
  const metadata = managementMetadata(schemaName, managementName);
  await createFileWithLogging(
    managementMetadataFilePath,
    JSON.stringify(metadata, null, 2),
    logger,
  );

  // Create management func code file
  const managementCodeFilePath = project.managementFuncCodePath(
    schemaName,
    managementName,
  );
  await ensureFileDoesNotExist(managementCodeFilePath, logger);
  await createFileWithLogging(
    managementCodeFilePath,
    managementCodeTemplate(schemaName, managementName),
    logger,
  );

  return {
    metadataPath: managementMetadataFilePath,
    codePath: managementCodeFilePath,
  };
}

/**
 * Generates a qualification function for a schema.
 *
 * Creates both the TypeScript code file and metadata JSON file for a
 * qualification function. Qualification functions validate and check conditions
 * for schema instances.
 *
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 * @param qualificationName - Name of the qualification function
 * @returns Object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists
 */
export async function generateSchemaQualification(
  ctx: Context,
  project: Project,
  schemaName: string,
  qualificationName: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  const logger = ctx.logger;

  // Create qualification func metadata file
  const qualificationMetadataFilePath = project.qualificationFuncMetadataPath(
    schemaName,
    qualificationName,
  );
  await ensureFileDoesNotExist(qualificationMetadataFilePath, logger);
  const metadata = createQualificationMetadata(schemaName, qualificationName);
  await createFileWithLogging(
    qualificationMetadataFilePath,
    JSON.stringify(metadata, null, 2),
    logger,
  );

  // Create qualification func code file
  const qualificationCodeFilePath = project.qualificationFuncCodePath(
    schemaName,
    qualificationName,
  );
  await ensureFileDoesNotExist(qualificationCodeFilePath, logger);
  await createFileWithLogging(
    qualificationCodeFilePath,
    qualificationCodeTemplate(schemaName, qualificationName),
    logger,
  );

  return {
    metadataPath: qualificationMetadataFilePath,
    codePath: qualificationCodeFilePath,
  };
}

export interface SchemaMetadata {
  /** The name of the schema */
  name: string;
  /** The category this schema belongs to */
  category: string;
  /** Optional description of the schema */
  description: string;
  /** Optional documentation link for the schema */
  documentation: string;
}

interface FunctionMetadata {
  /** The name of the function */
  name: string;
  /** The display name of the function */
  displayName: string;
  /** Optional description of the function */
  description: string;
}

function schemaMetadata(schemaName: string): SchemaMetadata {
  return {
    name: schemaName,
    category: "",
    description: "optional",
    documentation: "optional, should be a link",
  };
}

function actionMetadata(
  schemaName: string,
  actionName: string,
): FunctionMetadata {
  const name = `${schemaName}-action-${actionName}`;

  return {
    name,
    displayName: name,
    description: "optional description",
  };
}

function codegenMetadata(
  schemaName: string,
  codegenName: string,
): FunctionMetadata {
  const name = `${schemaName}-codegen-${codegenName}`;

  return {
    name,
    displayName: name,
    description: "optional description",
  };
}

function managementMetadata(
  schemaName: string,
  managementName: string,
): FunctionMetadata {
  const name = `${schemaName}-management-${managementName}`;

  return {
    name,
    displayName: name,
    description: "optional description",
  };
}

function createQualificationMetadata(
  schemaName: string,
  qualificationName: string,
): FunctionMetadata {
  const name = `${schemaName}-qualification-${qualificationName}`;

  return {
    name,
    displayName: name,
    description: "optional description",
  };
}

function schemaCodeTemplate(_schemaName: string): string {
  return `function main() {
  return new AssetBuilder().build();
}`;
}

function actionCodeTemplate(schemaName: string, actionName: string): string {
  return `function main(input: Input) {
  return {
    status: "error",
    message: "${schemaName} ${actionName} action is not implemented"
  }
}`;
}

function codegenCodeTemplate(
  _schemaName: string,
  _codegenName: string,
): string {
  return `function main() {
  const code = {};
  return {
    format: "json",
    code: JSON.stringify(code, null, 2),
  };
}`;
}

function managementCodeTemplate(
  _schemaName: string,
  _managementName: string,
): string {
  return `function main() {
  const ops = {
    update: {},
    actions: {
      self: {
        remove: [] as string[],
        add: [] as string[],
      },
    },
  };

  return {
    status: "ok",
    message: "Imported Resource",
    ops,
  };
}`;
}

function qualificationCodeTemplate(
  schemaName: string,
  qualificationName: string,
): string {
  return `function main(input: Input) {
  return {
    result: "failure",
    message: "${schemaName} ${qualificationName} qualification is not implemented"
  }
}`;
}

/**
 * Ensures that a file does not exist at the given path.
 * Throws FileExistsError if the file already exists.
 */
async function ensureFileDoesNotExist(
  filePath: AbsoluteFilePath,
  logger: Logger,
): Promise<void> {
  if (await filePath.exists()) {
    logger.error("File already exists at {filePath}", {
      filePath: filePath.toString(),
    });
    throw new FileExistsError(filePath.toString());
  }
}

/**
 * Creates a file with the given content and logs the creation.
 */
async function createFileWithLogging(
  filePath: AbsoluteFilePath,
  content: string,
  logger: Logger,
) {
  await filePath.writeTextFile(content);
  logger.info("Created: {filePath}", {
    filePath: filePath.toString(),
  });
}

/**
 * Error thrown when attempting to create a file that already exists.
 */
export class FileExistsError extends Error {
  constructor(public readonly path: string) {
    super(`File already exists at: ${path}`);
    this.name = "FileExistsError";
  }
}

/**
 * Error thrown when attempting to create a schema in a directory that already exists.
 */
export class DirectoryExistsError extends Error {
  constructor(public readonly path: string) {
    super(`Directory already exists at: ${path}`);
    this.name = "DirectoryExistsError";
  }
}
