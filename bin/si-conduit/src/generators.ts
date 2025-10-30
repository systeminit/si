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

import { SCHEMA_FILE_FORMAT_VERSION } from "./config.ts";
import * as materialize from "./materialize.ts";
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
export async function generateSchemaBase(project: Project, schemaName: string) {
  return await materialize.materializeSchemaBase(project, schemaName);
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
  project: Project,
  schemaName: string,
) {
  return await materialize.materializeSchemaActionBase(project, schemaName);
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
  project: Project,
  schemaName: string,
) {
  return await materialize.materializeSchemaCodegenBase(project, schemaName);
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
  project: Project,
  schemaName: string,
) {
  return await materialize.materializeSchemaManagementBase(project, schemaName);
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
  project: Project,
  schemaName: string,
) {
  return await materialize.materializeSchemaQualificationBase(
    project,
    schemaName,
  );
}

/**
 * Generates the authentication functions directory for a schema.
 *
 * Creates the authentication directory if it doesn't already exist. This
 * directory will contain authentication function files.
 *
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 */
export async function generateSchemaAuthBase(
  project: Project,
  schemaName: string,
) {
  return await materialize.materializeSchemaAuthBase(project, schemaName);
}

export async function generateSchemaFormatVersion(
  project: Project,
  schemaName: string,
): Promise<{ formatVersionPath: AbsoluteFilePath }> {
  const formatVersionBody = SCHEMA_FILE_FORMAT_VERSION.toString();

  return await materialize.materializeSchemaFormatVersion(
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
 * @param ctx - Application context for logging
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 * @returns Object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists
 */
export async function generateSchema(
  project: Project,
  schemaName: string,
): Promise<{ metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }> {
  const metadata = schemaMetadata(schemaName);
  const metadataBody = JSON.stringify(metadata, null, 2);
  const codeBody = schemaCodeTemplate(schemaName);

  return await materialize.materializeSchema(
    project,
    schemaName,
    metadataBody,
    codeBody,
  );
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
  project: Project,
  schemaName: string,
  actionName: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  const metadata = actionMetadata(schemaName, actionName);
  const metadataBody = JSON.stringify(metadata, null, 2);
  const codeBody = actionCodeTemplate(schemaName, actionName);

  return await materialize.materializeSchemaAction(
    project,
    schemaName,
    actionName,
    metadataBody,
    codeBody,
  );
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
  project: Project,
  schemaName: string,
  codegenName: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  const metadata = codegenMetadata(schemaName, codegenName);
  const metadataBody = JSON.stringify(metadata, null, 2);
  const codeBody = codegenCodeTemplate(schemaName, codegenName);

  return await materialize.materializeSchemaCodegen(
    project,
    schemaName,
    codegenName,
    metadataBody,
    codeBody,
  );
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
  project: Project,
  schemaName: string,
  managementName: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  const metadata = managementMetadata(schemaName, managementName);
  const metadataBody = JSON.stringify(metadata, null, 2);
  const codeBody = managementCodeTemplate(schemaName, managementName);

  return await materialize.materializeSchemaManagement(
    project,
    schemaName,
    managementName,
    metadataBody,
    codeBody,
  );
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
  project: Project,
  schemaName: string,
  qualificationName: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  const metadata = createQualificationMetadata(schemaName, qualificationName);
  const metadataBody = JSON.stringify(metadata, null, 2);
  const codeBody = qualificationCodeTemplate(schemaName, qualificationName);

  return await materialize.materializeSchemaQualification(
    project,
    schemaName,
    qualificationName,
    metadataBody,
    codeBody,
  );
}

/**
 * Generates an authentication function for a schema.
 *
 * Creates both the TypeScript code file and metadata JSON file for an
 * authentication function. Authentication functions handle credential validation
 * and authentication flows for resources.
 *
 * @param project - Project instance containing path utilities
 * @param schemaName - Name of the schema
 * @param authName - Name of the authentication function
 * @returns Object containing paths to the created metadata and code files
 * @throws {FileExistsError} If either file already exists
 */
export async function generateSchemaAuth(
  project: Project,
  schemaName: string,
  authName: string,
): Promise<{
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}> {
  const metadata = authMetadata(schemaName, authName);
  const metadataBody = JSON.stringify(metadata, null, 2);
  const codeBody = authCodeTemplate(schemaName, authName);

  return await materialize.materializeSchemaAuth(
    project,
    schemaName,
    authName,
    metadataBody,
    codeBody,
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
