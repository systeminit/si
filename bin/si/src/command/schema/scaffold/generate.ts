import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { MaterializableEntity } from "../../../materialize.ts";
import { getLogger } from "../../../logger.ts";
import type { AbsoluteFilePath } from "../../../project.ts";
import { Project } from "../../../project.ts";

const logger = getLogger();

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
