import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { getLogger } from "../../../logger.ts";
import type { AbsoluteFilePath } from "../../../project.ts";
import { Project } from "../../../project.ts";

const logger = getLogger();

export async function callSchemaScaffoldGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
): Promise<SchemaScaffoldResult> {
  logger.info("Generating scaffold for {schemaName}", {
    schemaName,
  });

  // Check that the schema directory doesn't exist
  const schemaBasePath = project.schemaBasePath(schemaName);
  if (await schemaBasePath.exists()) {
    logger.error("Directory already exists at {schemaBasePath}", {
      schemaBasePath: schemaBasePath.toString(),
    });
    throw new DirectoryExistsError(schemaBasePath.toString());
  }

  await generator.generateSchemaBase(project, schemaName);
  const { formatVersionPath } = await generator.generateSchemaFormatVersion(
    project,
    schemaName,
  );
  const { metadataPath, codePath } = await generator.generateSchema(
    project,
    schemaName,
  );

  await generator.generateSchemaActionBase(project, schemaName);
  const actionPaths = [];
  for (const actionName of Project.DEFAULT_ACTION_NAMES) {
    const paths = await generator.generateSchemaAction(
      project,
      schemaName,
      actionName,
    );
    actionPaths.push(paths);
  }

  await generator.generateSchemaCodegenBase(project, schemaName);
  const codegenPaths = [];
  for (const codegenName of Project.DEFAULT_CODEGEN_NAMES) {
    const paths = await generator.generateSchemaCodegen(
      project,
      schemaName,
      codegenName,
    );
    codegenPaths.push(paths);
  }

  await generator.generateSchemaManagementBase(project, schemaName);
  const managementPaths = [];
  for (const managementName of Project.DEFAULT_MANAGEMENT_NAMES) {
    const paths = await generator.generateSchemaManagement(
      project,
      schemaName,
      managementName,
    );
    managementPaths.push(paths);
  }

  await generator.generateSchemaQualificationBase(project, schemaName);
  const qualificationPaths = [];
  for (const qualificationName of Project.DEFAULT_QUALIFICATION_NAMES) {
    const paths = await generator.generateSchemaQualification(
      project,
      schemaName,
      qualificationName,
    );
    qualificationPaths.push(paths);
  }

  logger.info("Successfully generated scaffold for schema {schemaName}", {
    schemaName,
  });

  ctx.analytics.trackEvent("generate_scaffold", { schemaName: schemaName });

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
