import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { Project } from "../../../project.ts";
import type { AbsoluteFilePath } from "../../../project.ts";

export async function callSchemaManagementGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
  managementName: string,
): Promise<GeneratorResult> {
  const logger = ctx.logger;

  logger.info(
    "Generating management function {managementName} for {schemaName}",
    {
      schemaName,
      managementName,
    },
  );

  await generator.generateSchemaManagementBase(ctx, project, schemaName);

  const paths = await generator.generateSchemaManagement(
    ctx,
    project,
    schemaName,
    managementName,
  );

  logger.info(
    "Successfully generated management function {managementName} for schema {schemaName}",
    {
      schemaName,
      managementName,
    },
  );

  ctx.analytics.trackEvent("generate_management_function", {
    schemaName: schemaName,
    managementName: managementName,
  });

  return paths;
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}
