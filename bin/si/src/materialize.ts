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
import { AbsoluteDirectoryPath, FunctionKind, Project } from "./project.ts";
import type { AbsoluteFilePath } from "./project.ts";

const logger = getLogger();

export enum MaterializableEntity {
  Action = "action",
  Auth = "auth",
  Codegen = "codegen",
  Management = "management",
  Qualification = "qualification",
  Schema = "schema",
}

function materializableEntityToFunctionKind(entity: MaterializableEntity) {
  switch (entity) {
    case MaterializableEntity.Action:
      return FunctionKind.Action;
    case MaterializableEntity.Auth:
      return FunctionKind.Auth;
    case MaterializableEntity.Codegen:
      return FunctionKind.Codegen;
    case MaterializableEntity.Management:
      return FunctionKind.Management;
    case MaterializableEntity.Qualification:
      return FunctionKind.Qualification;
    default:
      throw new Error(`Can't make entity ${entity} a function kind`);
  }
}

export function functionKindToMaterializableEntity(kind: FunctionKind) {
  switch (kind) {
    case FunctionKind.Auth:
      return MaterializableEntity.Auth;
    case FunctionKind.Codegen:
      return MaterializableEntity.Codegen;
    case FunctionKind.Management:
      return MaterializableEntity.Management;
    case FunctionKind.Qualification:
      return MaterializableEntity.Qualification;
    case FunctionKind.Action:
      return MaterializableEntity.Action;
    default:
      throw new Error(
        `Can't make function kind ${kind} a materializable entity`,
      );
  }
}

/**
 * Creates the base directory for an entity if it doesn't already exist.
 *
 * This is a unified function for creating base directories for schemas and
 * their various function types (actions, auth, codegens, management,
 * qualifications).
 *
 * @param project - The project instance containing path configuration
 * @param entity - The type of entity as defined in MaterializableEntity
 * @param name - The name of the schema
 * @param isOverlay - should entity be created in overlays directory or schemas directory
 */
export async function materializeEntityBase(
  project: Project,
  entity: MaterializableEntity,
  name: string,
  isOverlay?: boolean,
) {
  const module = isOverlay ? project.overlays : project.schemas;

  let basePath: AbsoluteDirectoryPath;
  if (entity === MaterializableEntity.Schema) {
    basePath = module.schemaBasePath(name);
  } else {
    const funcKind = materializableEntityToFunctionKind(entity);
    basePath = module.funcBasePath(name, funcKind);
  }

  // Create base directory
  if (!(await basePath.exists())) {
    await basePath.mkdir({ recursive: true });
    logger.info("  - Created directory: {basePath}", {
      basePath: basePath.relativeToStr(project),
    });
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
  const formatVersionFilePath = project.schemas.formatVersionPath(schemaName);
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

type EntityName = {
  entity: MaterializableEntity.Schema;
  name: string;
} | {
  entity: Exclude<MaterializableEntity, MaterializableEntity.Schema>;
  schemaName: string;
  name: string;
};

/**
 * Materializes entity files (metadata and code) for any entity type.
 *
 * This is a unified function that handles all entity types including schemas,
 * actions, auth, codegens, management functions, and qualifications. It creates
 * both the metadata JSON file and the TypeScript code file for the entity.
 *
 * @param project - The project instance containing path configuration
 * @param entityName - The entity identifier containing the type and names
 * @param metadataBody - The JSON metadata content for the entity
 * @param codeBody - The TypeScript code for the entity
 * @param options - Optional configuration including overwrite flag
 * @returns An object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists and overwrite is not enabled
 */
export async function materializeEntity(
  project: Project,
  entityName: EntityName,
  metadataBody: string,
  codeBody: string,
  options?: {
    overwrite?: boolean;
    isOverlay?: boolean;
  },
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  const isOverlay = options?.isOverlay ?? false;

  let metadataFilePath: AbsoluteFilePath;
  let codeFilePath: AbsoluteFilePath;

  if (entityName.entity === MaterializableEntity.Schema) {
    if (options?.isOverlay) {
      throw new Error("Can't materialize schema in overlays directory");
    }

    metadataFilePath = project.schemas.schemaMetadataPath(entityName.name);
    codeFilePath = project.schemas.schemaFuncCodePath(entityName.name);
  } else {
    const module = isOverlay ? project.overlays : project.schemas;
    const funcKind = materializableEntityToFunctionKind(entityName.entity);
    metadataFilePath = module.funcMetadataPath(
      entityName.schemaName,
      entityName.name,
      funcKind,
    );
    codeFilePath = module.funcCodePath(
      entityName.schemaName,
      entityName.name,
      funcKind,
    );
  }

  if (!options?.overwrite) {
    await ensureFileDoesNotExist(metadataFilePath);
  }
  await createFileWithLogging(project, metadataFilePath, metadataBody);

  if (!options?.overwrite) {
    await ensureFileDoesNotExist(codeFilePath);
  }
  await createFileWithLogging(project, codeFilePath, codeBody);

  return {
    metadataPath: metadataFilePath,
    codePath: codeFilePath,
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
