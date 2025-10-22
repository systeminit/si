import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { getLogger } from "../../../logger.ts";
import { Project } from "../../../project.ts";
import type { AbsoluteFilePath } from "../../../project.ts";

const logger = getLogger();

export async function callSchemaQualificationGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
  qualificationName: string,
): Promise<GeneratorResult> {
  logger.info(
    "Generating qualification function {qualificationName} for {schemaName}",
    {
      schemaName,
      qualificationName,
    },
  );

  await generator.generateSchemaQualificationBase(project, schemaName);

  const paths = await generator.generateSchemaQualification(
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
