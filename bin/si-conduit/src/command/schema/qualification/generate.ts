import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { Project } from "../../../project.ts";
import type { AbsoluteFilePath } from "../../../project.ts";

export async function callSchemaQualificationGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
  qualificationName: string,
): Promise<GeneratorResult> {
  const logger = ctx.logger;

  logger.info(
    "Generating qualification function {qualificationName} for {schemaName}",
    {
      schemaName,
      qualificationName,
    },
  );

  await generator.generateSchemaQualificationBase(ctx, project, schemaName);

  const paths = await generator.generateSchemaQualification(
    ctx,
    project,
    schemaName,
    qualificationName,
  );

  logger.info(
    "Successfully generated qualification function {qualificationName} for schema {schemaName}",
    {
      schemaName,
      qualificationName,
    },
  );

  ctx.analytics.trackEvent("generate_qualification_function", {
    schemaName: schemaName,
    qualificationName: qualificationName,
  });

  return paths;
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}
