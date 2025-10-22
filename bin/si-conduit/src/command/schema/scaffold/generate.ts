import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import type { AbsoluteFilePath } from "../../../project.ts";
import { Project } from "../../../project.ts";

export async function callSchemaScaffoldGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
): Promise<SchemaScaffoldResult> {
  const logger = ctx.logger;

  logger.info("Generating scaffold for {schemaName}", {
    schemaName,
  });

  const { formatVersionPath } = await generator.generateSchemaBase(
    ctx,
    project,
    schemaName,
  );
  const { metadataPath, codePath } = await generator.generateSchema(
    ctx,
    project,
    schemaName,
  );

  await generator.generateSchemaActionBase(ctx, project, schemaName);
  const actionPaths = [];
  for (const actionName of Project.DEFAULT_ACTION_NAMES) {
    const paths = await generator.generateSchemaAction(
      ctx,
      project,
      schemaName,
      actionName,
    );
    actionPaths.push(paths);
  }

  await generator.generateSchemaCodegenBase(ctx, project, schemaName);
  const codegenPaths = [];
  for (const codegenName of Project.DEFAULT_CODEGEN_NAMES) {
    const paths = await generator.generateSchemaCodegen(
      ctx,
      project,
      schemaName,
      codegenName,
    );
    codegenPaths.push(paths);
  }

  await generator.generateSchemaManagementBase(ctx, project, schemaName);
  const managementPaths = [];
  for (const managementName of Project.DEFAULT_MANAGEMENT_NAMES) {
    const paths = await generator.generateSchemaManagement(
      ctx,
      project,
      schemaName,
      managementName,
    );
    managementPaths.push(paths);
  }

  await generator.generateSchemaQualificationBase(ctx, project, schemaName);
  const qualificationPaths = [];
  for (const qualificationName of Project.DEFAULT_QUALIFICATION_NAMES) {
    const paths = await generator.generateSchemaQualification(
      ctx,
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
