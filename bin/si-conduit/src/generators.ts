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
 * import { generateSchemaFunction, MaterializableEntity } from "./generators.ts";
 * import { Project } from "./project.ts";
 *
 * const project = new Project("/path/to/project");
 *
 * // Generate an action function
 * await generateSchemaFunction(project, "MySchema", "create", MaterializableEntity.Action, false);
 * ```
 *
 * @module
 */

import { SCHEMA_FILE_FORMAT_VERSION } from "./config.ts";
import * as materialize from "./materialize.ts";
import { Project } from "./project.ts";
import type { AbsoluteFilePath } from "./project.ts";
import { MaterializableEntity } from "./materialize.ts";

/**
 * Generates the base directory structure for an entity.
 *
 * Creates the appropriate directory for the given entity type. This is a
 * unified function that handles all entity types including schemas and their
 * various function directories.
 *
 * @param project - Project instance containing path utilities
 * @param entity - The type of entity (Schema, Action, Auth, Codegen, Management, Qualification)
 * @param schemaName - Name of the schema
 * @param isOverlay
 */
export function generateEntityBase(
  project: Project,
  entity: MaterializableEntity,
  schemaName: string,
  isOverlay: boolean,
) {
  return materialize.materializeEntityBase(
    project,
    entity,
    schemaName,
    isOverlay,
  );
}

export async function generateSchemaFormatVersion(
  project: Project,
  schemaName: string,
) {
  const formatVersionBody = SCHEMA_FILE_FORMAT_VERSION.toString();

  return materialize.materializeSchemaFormatVersion(
    project,
    schemaName,
    formatVersionBody,
  );
}

/**
 * Generates schema metadata and code files.
 *
 * Creates the schema.metadata.json and schema.ts files for a schema. The
 * metadata file contains schema information like name, category, and
 * description. The code file contains the schema definition template.
 *
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 * @param isOverlay
 * @returns Object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists
 */
export async function generateSchema(
  project: Project,
  schemaName: string,
  isOverlay: boolean,
): Promise<{ metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }> {
  const metadata = schemaMetadata(schemaName);
  const metadataBody = JSON.stringify(metadata, null, 2);
  const codeBody = schemaCodeTemplate(schemaName);

  return await materialize.materializeEntity(
    project,
    { entity: MaterializableEntity.Schema, name: schemaName },
    metadataBody,
    codeBody,
    { isOverlay },
  );
}

export async function generateSchemaFunction(
  project: Project,
  schemaName: string,
  funcName: string,
  entity: MaterializableEntity,
  isOverlay: boolean,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  let metadataGenerator: (
    schemaName: string,
    funcName: string,
  ) => FunctionMetadata;
  let codeGenerator: (schemaName: string, funcName: string) => string;

  switch (entity) {
    case MaterializableEntity.Action:
      metadataGenerator = actionMetadata;
      codeGenerator = actionCodeTemplate;
      break;
    case MaterializableEntity.Auth:
      metadataGenerator = authMetadata;
      codeGenerator = authCodeTemplate;
      break;
    case MaterializableEntity.Codegen:
      metadataGenerator = codegenMetadata;
      codeGenerator = codegenCodeTemplate;
      break;
    case MaterializableEntity.Management:
      metadataGenerator = managementMetadata;
      codeGenerator = managementCodeTemplate;
      break;
    case MaterializableEntity.Qualification:
      metadataGenerator = qualificationMetadata;
      codeGenerator = qualificationCodeTemplate;
      break;
    default:
      throw new Error(`Can't make entity ${entity} a function kind`);
  }

  const metadata = metadataGenerator(schemaName, funcName);
  const metadataBody = JSON.stringify(metadata, null, 2);
  const codeBody = codeGenerator(schemaName, funcName);

  return await materialize.materializeEntity(
    project,
    { entity, schemaName, name: funcName },
    metadataBody,
    codeBody,
    { isOverlay },
  );
}

export interface SchemaMetadata {
  /** The name of the schema */
  name: string;
  /** The category this schema belongs to */
  category: string;
  /** Optional description of the schema */
  description?: string | null;
  /** Optional documentation link for the schema */
  documentation?: string | null;
}

export interface FunctionMetadata {
  /** The name of the function */
  name: string;
  /** The display name of the function */
  displayName?: string | null;
  /** Optional description of the function */
  description?: string | null;
}

function schemaMetadata(schemaName: string): SchemaMetadata {
  return {
    name: schemaName,
    category: "",
    description: "optional",
    documentation: null, // "optional, should be a link",
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

function qualificationMetadata(
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

function authMetadata(schemaName: string, authName: string): FunctionMetadata {
  const name = `${schemaName}-auth-${authName}`;

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

function authCodeTemplate(schemaName: string, authName: string): string {
  return `function main(input: Input) {
  throw new Error(
    "${schemaName} ${authName} authentication function unimplemented!"
  );
}`;
}
