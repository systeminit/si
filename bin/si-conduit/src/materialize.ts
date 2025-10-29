/**
 * Materialize Module - File System Operations for Schema Materialization
 *
 * This module provides functions to create and write schema files, metadata,
 * and function code to the file system. It is primarily used when pulling
 * schemas from remote workspaces or generating new schema structures locally.
 *
 * ## Key Responsibilities
 *
 * - Creating directory structures for schemas and their functions
 * - Writing schema metadata and code files to disk
 * - Writing function (action, auth, codegen, management, qualification) files
 * - Ensuring files don't already exist before creation
 * - Logging file creation operations
 *
 * ## Usage Pattern
 *
 * Functions in this module follow a consistent pattern:
 * 1. Check if directories need to be created
 * 2. Ensure target files don't already exist (throws FileExistsError if they
 *    do)
 * 3. Write files to disk
 * 4. Log the creation operations
 *
 * @module
 */

import { getLogger } from "./logger.ts";
import { Project } from "./project.ts";
import type { AbsoluteFilePath } from "./project.ts";

const logger = getLogger();

/**
 * Creates the base directory for a schema if it doesn't already exist.
 *
 * This is typically the first operation when materializing a schema, creating
 * the `schemas/<schema-name>/` directory.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema to create a directory for
 *
 * @example
 * ```ts
 * await materializeSchemaBase(project, "MyAwsResource");
 * // Creates: schemas/MyAwsResource/
 * ```
 */
export async function materializeSchemaBase(
  project: Project,
  schemaName: string,
) {
  const schemaBasePath = project.schemaBasePath(schemaName);

  // Create schema base directory
  if (!(await schemaBasePath.exists())) {
    await schemaBasePath.mkdir({ recursive: true });
    logger.info("  - Created schema directory: {schemaBasePath}", {
      schemaBasePath: schemaBasePath.relativeToStr(project),
    });
  }
}

/**
 * Creates the actions directory for a schema if it doesn't already exist.
 *
 * This creates the `schemas/<schema-name>/actions/` directory where action
 * function files (create, destroy, refresh, update) will be stored.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 *
 * @example
 * ```ts
 * await materializeSchemaActionBase(project, "MyAwsResource");
 * // Creates: schemas/MyAwsResource/actions/
 * ```
 */
export async function materializeSchemaActionBase(
  project: Project,
  schemaName: string,
) {
  const actionBasePath = project.actionBasePath(schemaName);

  // Create the action base directory
  if (!(await actionBasePath.exists())) {
    await actionBasePath.mkdir({ recursive: true });
    logger.info("  - Created actions directory: {actionBasePath}", {
      actionBasePath: actionBasePath.relativeToStr(project),
    });
  }
}

/**
 * Creates the authentication directory for a schema if it doesn't already
 * exist.
 *
 * This creates the `schemas/<schema-name>/auth/` directory where authentication
 * function files will be stored.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 *
 * @example
 * ```ts
 * await materializeSchemaAuthBase(project, "MyAwsResource");
 * // Creates: schemas/MyAwsResource/auth/
 * ```
 */
export async function materializeSchemaAuthBase(
  project: Project,
  schemaName: string,
) {
  const authBasePath = project.authBasePath(schemaName);

  // Create the auth base directory
  if (!(await authBasePath.exists())) {
    await authBasePath.mkdir({ recursive: true });
    logger.info("  - Created authentication directory: {authBasePath}", {
      authBasePath: authBasePath.relativeToStr(project),
    });
  }
}

/**
 * Creates the code generators directory for a schema if it doesn't already
 * exist.
 *
 * This creates the `schemas/<schema-name>/codeGenerators/` directory where
 * code generation function files will be stored.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 *
 * @example
 * ```ts
 * await materializeSchemaCodegenBase(project, "MyAwsResource");
 * // Creates: schemas/MyAwsResource/codeGenerators/
 * ```
 */
export async function materializeSchemaCodegenBase(
  project: Project,
  schemaName: string,
) {
  const codegenBasePath = project.codegenBasePath(schemaName);

  // Create the codegen base directory
  if (!(await codegenBasePath.exists())) {
    await codegenBasePath.mkdir({ recursive: true });
    logger.info("  - Created code generators directory: {codegenBasePath}", {
      codegenBasePath: codegenBasePath.relativeToStr(project),
    });
  }
}

/**
 * Creates the management directory for a schema if it doesn't already exist.
 *
 * This creates the `schemas/<schema-name>/management/` directory where
 * management function files will be stored.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 *
 * @example
 * ```ts
 * await materializeSchemaManagementBase(project, "MyAwsResource");
 * // Creates: schemas/MyAwsResource/management/
 * ```
 */
export async function materializeSchemaManagementBase(
  project: Project,
  schemaName: string,
) {
  const managementBasePath = project.managementBasePath(schemaName);

  // Create the management base directory
  if (!(await managementBasePath.exists())) {
    await managementBasePath.mkdir({ recursive: true });
    logger.info("  - Created management directory: {managementBasePath}", {
      managementBasePath: managementBasePath.relativeToStr(project),
    });
  }
}

/**
 * Creates the qualifications directory for a schema if it doesn't already
 * exist.
 *
 * This creates the `schemas/<schema-name>/qualifications/` directory where
 * qualification function files will be stored.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 *
 * @example
 * ```ts
 * await materializeSchemaQualificationBase(project, "MyAwsResource");
 * // Creates: schemas/MyAwsResource/qualifications/
 * ```
 */
export async function materializeSchemaQualificationBase(
  project: Project,
  schemaName: string,
) {
  const qualificationBasePath = project.qualificationBasePath(schemaName);

  // Create the qualification base directory
  if (!(await qualificationBasePath.exists())) {
    await qualificationBasePath.mkdir({ recursive: true });
    logger.info(
      "  - Created qualifications directory: {qualificationBasePath}",
      {
        qualificationBasePath: qualificationBasePath.relativeToStr(project),
      },
    );
  }
}

/**
 * Materializes the format version file for a schema.
 *
 * Creates the `.format-version` file that indicates the schema file format
 * version. This file is used to ensure compatibility when reading and writing
 * schema files.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 * @param formatVersionBody - The format version content (typically a version
 * number)
 * @param options - Optional configuration including overwrite flag
 * @returns An object containing the path to the created format version file
 * @throws {FileExistsError} If the format version file already exists and
 * overwrite is not enabled
 *
 * @example
 * ```ts
 * const result = await materializeSchemaFormatVersion(
 *   project,
 *   "MyAwsResource",
 *   "1",
 * );
 * // Creates: schemas/MyAwsResource/.format-version
 * console.log(result.formatVersionPath);
 * ```
 */
export async function materializeSchemaFormatVersion(
  project: Project,
  schemaName: string,
  formatVersionBody: string,
  options?: { overwrite?: boolean },
): Promise<{ formatVersionPath: AbsoluteFilePath }> {
  // Create the format version file
  const formatVersionFilePath = project.schemaFormatVersionPath(schemaName);
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(formatVersionFilePath);
  }
  await createFileWithLogging(
    project,
    formatVersionFilePath,
    formatVersionBody,
  );

  return {
    formatVersionPath: formatVersionFilePath,
  };
}

/**
 * Materializes the main schema files (metadata and code).
 *
 * Creates both the schema metadata JSON file and the schema TypeScript code
 * file. These files define the schema's properties, structure, and asset
 * function.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 * @param metadataBody - The JSON metadata content for the schema
 * @param codeBody - The TypeScript code for the schema's asset function
 * @param options - Optional configuration including overwrite flag
 * @returns An object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists and overwrite is not
 * enabled
 *
 * @example
 * ```ts
 * const metadata = JSON.stringify({
 *   name: "MyAwsResource",
 *   category: "AWS",
 *   description: "An AWS resource"
 * });
 * const code = "export default function() { ... }";
 *
 * const result = await materializeSchema(
 *   project,
 *   "MyAwsResource",
 *   metadata,
 *   code,
 * );
 * // Creates:
 * //   schemas/MyAwsResource/schema.metadata.json
 * //   schemas/MyAwsResource/schema.ts
 * ```
 */
export async function materializeSchema(
  project: Project,
  schemaName: string,
  metadataBody: string,
  codeBody: string,
  options?: { overwrite?: boolean },
): Promise<{ metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }> {
  // Create schema metadata file
  const metadataFilePath = project.schemaMetadataPath(schemaName);
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(metadataFilePath);
  }
  await createFileWithLogging(project, metadataFilePath, metadataBody);

  // Create schema func code file
  const schemaCodeFilePath = project.schemaFuncCodePath(schemaName);
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(schemaCodeFilePath);
  }
  await createFileWithLogging(project, schemaCodeFilePath, codeBody);

  return {
    metadataPath: metadataFilePath,
    codePath: schemaCodeFilePath,
  };
}

/**
 * Materializes an action function for a schema.
 *
 * Creates both the action metadata JSON file and the action TypeScript code
 * file. Actions are functions that perform operations on resources (e.g.,
 * create, destroy, refresh, update).
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 * @param actionName - The name of the action (e.g., "create", "destroy")
 * @param metadataBody - The JSON metadata content for the action
 * @param codeBody - The TypeScript code for the action function
 * @param options - Optional configuration including overwrite flag
 * @returns An object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists and overwrite is not
 * enabled
 *
 * @example
 * ```ts
 * const metadata = JSON.stringify({
 *   name: "create",
 *   displayName: "Create Resource"
 * });
 * const code = "export default async function() { ... }";
 *
 * const result = await materializeSchemaAction(
 *   project,
 *   "MyAwsResource",
 *   "create",
 *   metadata,
 *   code
 * );
 * // Creates:
 * //   schemas/MyAwsResource/actions/create.metadata.json
 * //   schemas/MyAwsResource/actions/create.ts
 * ```
 */
export async function materializeSchemaAction(
  project: Project,
  schemaName: string,
  actionName: string,
  metadataBody: string,
  codeBody: string,
  options?: { overwrite?: boolean },
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  // Create action func metadata file
  const actionMetadataFilePath = project.actionFuncMetadataPath(
    schemaName,
    actionName,
  );
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(actionMetadataFilePath);
  }
  await createFileWithLogging(project, actionMetadataFilePath, metadataBody);

  // Create action func code file
  const actionCodeFilePath = project.actionFuncCodePath(schemaName, actionName);
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(actionCodeFilePath);
  }
  await createFileWithLogging(project, actionCodeFilePath, codeBody);

  return {
    metadataPath: actionMetadataFilePath,
    codePath: actionCodeFilePath,
  };
}

/**
 * Materializes an authentication function for a schema.
 *
 * Creates both the authentication metadata JSON file and the authentication
 * TypeScript code file. Authentication functions handle credential validation
 * and authentication flows for resources.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 * @param authName - The name of the authentication function
 * @param metadataBody - The JSON metadata content for the authentication
 * function
 * @param codeBody - The TypeScript code for the authentication function
 * @param options - Optional configuration including overwrite flag
 * @returns An object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists and overwrite is not
 * enabled
 *
 * @example
 * ```ts
 * const metadata = JSON.stringify({
 *   name: "oauth",
 *   displayName: "OAuth Authentication"
 * });
 * const code = "export default async function() { ... }";
 *
 * const result = await materializeSchemaAuth(
 *   project,
 *   "MyAwsResource",
 *   "oauth",
 *   metadata,
 *   code,
 *   { overwrite: true }
 * );
 * // Creates:
 * //   schemas/MyAwsResource/auth/oauth.metadata.json
 * //   schemas/MyAwsResource/auth/oauth.ts
 * ```
 */
export async function materializeSchemaAuth(
  project: Project,
  schemaName: string,
  authName: string,
  metadataBody: string,
  codeBody: string,
  options?: { overwrite?: boolean },
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  // Create auth func metadata file
  const authMetadataFilePath = project.authFuncMetadataPath(
    schemaName,
    authName,
  );
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(authMetadataFilePath);
  }
  await createFileWithLogging(project, authMetadataFilePath, metadataBody);

  // Create auth func code file
  const authCodeFilePath = project.authFuncCodePath(schemaName, authName);
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(authCodeFilePath);
  }
  await createFileWithLogging(project, authCodeFilePath, codeBody);

  return {
    metadataPath: authMetadataFilePath,
    codePath: authCodeFilePath,
  };
}

/**
 * Materializes a code generation function for a schema.
 *
 * Creates both the codegen metadata JSON file and the codegen TypeScript code
 * file. Code generators produce configuration files or infrastructure code
 * from schema component data.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 * @param codegenName - The name of the code generator
 * @param metadataBody - The JSON metadata content for the code generator
 * @param codeBody - The TypeScript code for the code generator function
 * @param options - Optional configuration including overwrite flag
 * @returns An object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists and overwrite is not
 * enabled
 *
 * @example
 * ```ts
 * const metadata = JSON.stringify({
 *   name: "terraform",
 *   displayName: "Terraform HCL Generator"
 * });
 * const code = "export default function() { return 'resource ...'; }";
 *
 * const result = await materializeSchemaCodegen(
 *   project,
 *   "MyAwsResource",
 *   "terraform",
 *   metadata,
 *   code
 * );
 * // Creates:
 * //   schemas/MyAwsResource/codeGenerators/terraform.metadata.json
 * //   schemas/MyAwsResource/codeGenerators/terraform.ts
 * ```
 */
export async function materializeSchemaCodegen(
  project: Project,
  schemaName: string,
  codegenName: string,
  metadataBody: string,
  codeBody: string,
  options?: { overwrite?: boolean },
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  // Create codegen func metadata file
  const codegenMetadataFilePath = project.codegenFuncMetadataPath(
    schemaName,
    codegenName,
  );
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(codegenMetadataFilePath);
  }
  await createFileWithLogging(project, codegenMetadataFilePath, metadataBody);

  // Create codegen func code file
  const codegenCodeFilePath = project.codegenFuncCodePath(
    schemaName,
    codegenName,
  );
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(codegenCodeFilePath);
  }
  await createFileWithLogging(project, codegenCodeFilePath, codeBody);

  return {
    metadataPath: codegenMetadataFilePath,
    codePath: codegenCodeFilePath,
  };
}

/**
 * Materializes a management function for a schema.
 *
 * Creates both the management metadata JSON file and the management TypeScript
 * code file. Management functions handle reconciliation and lifecycle
 * operations for resources.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 * @param managementName - The name of the management function
 * @param metadataBody - The JSON metadata content for the management function
 * @param codeBody - The TypeScript code for the management function
 * @param options - Optional configuration including overwrite flag
 * @returns An object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists and overwrite is not
 * enabled
 *
 * @example
 * ```ts
 * const metadata = JSON.stringify({
 *   name: "reconcile",
 *   displayName: "Reconcile Resource"
 * });
 * const code = "export default async function() { ... }";
 *
 * const result = await materializeSchemaManagement(
 *   project,
 *   "MyAwsResource",
 *   "reconcile",
 *   metadata,
 *   code
 * );
 * // Creates:
 * //   schemas/MyAwsResource/management/reconcile.metadata.json
 * //   schemas/MyAwsResource/management/reconcile.ts
 * ```
 */
export async function materializeSchemaManagement(
  project: Project,
  schemaName: string,
  managementName: string,
  metadataBody: string,
  codeBody: string,
  options?: { overwrite?: boolean },
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  // Create management func metadata file
  const managementMetadataFilePath = project.managementFuncMetadataPath(
    schemaName,
    managementName,
  );
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(managementMetadataFilePath);
  }
  await createFileWithLogging(
    project,
    managementMetadataFilePath,
    metadataBody,
  );

  // Create management func code file
  const managementCodeFilePath = project.managementFuncCodePath(
    schemaName,
    managementName,
  );
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(managementCodeFilePath);
  }
  await createFileWithLogging(project, managementCodeFilePath, codeBody);

  return {
    metadataPath: managementMetadataFilePath,
    codePath: managementCodeFilePath,
  };
}

/**
 * Materializes a qualification function for a schema.
 *
 * Creates both the qualification metadata JSON file and the qualification
 * TypeScript code file. Qualifications validate component state and properties
 * to ensure they meet required criteria.
 *
 * @param project - The project instance containing path configuration
 * @param schemaName - The name of the schema
 * @param qualificationName - The name of the qualification function
 * @param metadataBody - The JSON metadata content for the qualification
 * @param codeBody - The TypeScript code for the qualification function
 * @param options - Optional configuration including overwrite flag
 * @returns An object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists and overwrite is not
 * enabled
 *
 * @example
 * ```ts
 * const metadata = JSON.stringify({
 *   name: "validate",
 *   displayName: "Validate Configuration"
 * });
 * const code = "export default function() { return { status: 'ok' }; }";
 *
 * const result = await materializeSchemaQualification(
 *   project,
 *   "MyAwsResource",
 *   "validate",
 *   metadata,
 *   code
 * );
 * // Creates:
 * //   schemas/MyAwsResource/qualifications/validate.metadata.json
 * //   schemas/MyAwsResource/qualifications/validate.ts
 * ```
 */
export async function materializeSchemaQualification(
  project: Project,
  schemaName: string,
  qualificationName: string,
  metadataBody: string,
  codeBody: string,
  options?: { overwrite?: boolean },
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  // Create qualification func metadata file
  const qualificationMetadataFilePath = project.qualificationFuncMetadataPath(
    schemaName,
    qualificationName,
  );
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(qualificationMetadataFilePath);
  }
  await createFileWithLogging(
    project,
    qualificationMetadataFilePath,
    metadataBody,
  );

  // Create qualification func code file
  const qualificationCodeFilePath = project.qualificationFuncCodePath(
    schemaName,
    qualificationName,
  );
  if (!options?.overwrite) {
    await ensureFileDoesNotExist(qualificationCodeFilePath);
  }
  await createFileWithLogging(project, qualificationCodeFilePath, codeBody);

  return {
    metadataPath: qualificationMetadataFilePath,
    codePath: qualificationCodeFilePath,
  };
}

/**
 * Ensures that a file does not exist at the given path.
 * Throws FileExistsError if the file already exists.
 */
export async function ensureFileDoesNotExist(
  filePath: AbsoluteFilePath,
): Promise<void> {
  if (await filePath.exists()) {
    logger.error("File already exists at {filePath}", {
      filePath: filePath.toString(),
    });
    throw new FileExistsError(filePath.toString());
  }
}

/**
 * Creates or updates a file with the given content and logs the creation.
 *
 * Skips writing if the file already exists with identical content.
 */
export async function createFileWithLogging(
  project: Project,
  filePath: AbsoluteFilePath,
  content: string,
) {
  const alreadyExists = await filePath.exists();

  // If file exists, check if contents would change
  if (alreadyExists) {
    const existingContent = await filePath.readTextFile();
    if (existingContent === content) {
      // Contents are identical, skip writing
      logger.info(`  - Skipped (unchanged): {filePath}`, {
        filePath: filePath.relativeToStr(project),
      });
      return;
    }
  }

  await filePath.writeTextFile(content);
  const verb = alreadyExists ? "Updated" : "Created";
  logger.info(`  - ${verb}: {filePath}`, {
    filePath: filePath.relativeToStr(project),
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
