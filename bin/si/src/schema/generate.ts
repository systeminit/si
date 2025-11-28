/**
 * Schema Generation Commands
 *
 * This module provides functionality for generating schemas and their associated
 * functions (actions, authentication, codegen, management, qualification).
 *
 * @module
 */

import type { Context } from "../context.ts";
import * as generator from "./generators.ts";
import {
  functionKindToMaterializableEntity,
  MaterializableEntity,
} from "./materialize.ts";
import { getLogger } from "../logger.ts";
import { type FunctionKind, Project } from "./project.ts";
import type { AbsoluteFilePath } from "./project.ts";

const logger = getLogger();

/**
 * Generate a function for a schema (action, auth, codegen, management, or qualification)
 */
export async function callSchemaFuncGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
  funcKind: FunctionKind,
  name: string,
  isOverlay: boolean,
): Promise<GeneratorResult> {
  const overlayMsg = isOverlay ? " overlay" : "";

  logger.info(
    `Generating${overlayMsg} ${funcKind} function {name} for schema {schemaName}`,
    {
      overlayMsg,
      schemaName,
      name,
    },
  );
  logger.info("---");
  logger.info("");

  const entity = functionKindToMaterializableEntity(funcKind);

  await generator.generateEntityBase(project, entity, schemaName, isOverlay);

  const paths = await generator.generateSchemaFunction(
    project,
    schemaName,
    name,
    entity,
    isOverlay,
  );

  logger.info("");
  logger.info("---");
  logger.info(
    `Successfully generated${overlayMsg} ${funcKind} function for schema {schemaName}`,
    {
      overlayMsg,
      schemaName,
    },
  );
  logger.info("  - {name}", { name });

  const eventName = `schema_${entity}_generate`;

  ctx.analytics.trackEvent(eventName, {
    schemaName: schemaName,
    actionName: name,
  });

  return paths;
}

/**
 * Generate a complete schema scaffold with default functions
 */
export async function callSchemaScaffoldGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
): Promise<SchemaScaffoldResult> {
  logger.info("Generating scaffold for schema {schemaName}", {
    schemaName,
  });
  logger.info("---");
  logger.info("");

  // Check that the schema directory doesn't exist
  const schemaBasePath = project.schemas.schemaBasePath(schemaName);
  if (await schemaBasePath.exists()) {
    logger.error("Directory already exists at {schemaBasePath}", {
      schemaBasePath: schemaBasePath.toString(),
    });
    throw new DirectoryExistsError(schemaBasePath.toString());
  }

  await generator.generateEntityBase(
    project,
    MaterializableEntity.Schema,
    schemaName,
    false,
  );
  const { formatVersionPath } = await generator.generateSchemaFormatVersion(
    project,
    schemaName,
  );
  const { metadataPath, codePath } = await generator.generateSchema(
    project,
    schemaName,
    false,
  );

  await generator.generateEntityBase(
    project,
    MaterializableEntity.Action,
    schemaName,
    false,
  );
  const actionPaths = [];
  for (const actionName of Project.DEFAULT_ACTION_NAMES) {
    const paths = await generator.generateSchemaFunction(
      project,
      schemaName,
      actionName,
      MaterializableEntity.Action,
      false,
    );
    actionPaths.push(paths);
  }

  await generator.generateEntityBase(
    project,
    MaterializableEntity.Codegen,
    schemaName,
    false,
  );
  const codegenPaths = [];
  for (const codegenName of Project.DEFAULT_CODEGEN_NAMES) {
    const paths = await generator.generateSchemaFunction(
      project,
      schemaName,
      codegenName,
      MaterializableEntity.Codegen,
      false,
    );
    codegenPaths.push(paths);
  }

  await generator.generateEntityBase(
    project,
    MaterializableEntity.Management,
    schemaName,
    false,
  );
  const managementPaths = [];
  for (const managementName of Project.DEFAULT_MANAGEMENT_NAMES) {
    const paths = await generator.generateSchemaFunction(
      project,
      schemaName,
      managementName,
      MaterializableEntity.Management,
      false,
    );
    managementPaths.push(paths);
  }

  await generator.generateEntityBase(
    project,
    MaterializableEntity.Qualification,
    schemaName,
    false,
  );
  const qualificationPaths = [];
  for (const qualificationName of Project.DEFAULT_QUALIFICATION_NAMES) {
    const paths = await generator.generateSchemaFunction(
      project,
      schemaName,
      qualificationName,
      MaterializableEntity.Qualification,
      false,
    );
    qualificationPaths.push(paths);
  }

  logger.info("");
  logger.info("---");
  logger.info("Successfully generated scaffold for schema {schemaName}", {
    schemaName,
  });

  ctx.analytics.trackEvent("schema_scaffold_generate", {
    schemaName: schemaName,
  });

  return {
    formatVersionPath,
    metadataPath,
    codePath,
    actionPaths,
    codegenPaths,
    managementPaths,
    qualificationPaths,
  };
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}

/**
 * Result returned after successfully scaffolding a schema.
 */
export interface SchemaScaffoldResult {
  /** Path to the created format version file */
  formatVersionPath: AbsoluteFilePath;
  /** Path to the created metadata file */
  metadataPath: AbsoluteFilePath;
  /** Path to the created schema code file */
  codePath: AbsoluteFilePath;
  actionPaths: { metadataPath: AbsoluteFilePath; codePath: AbsoluteFilePath }[];
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
