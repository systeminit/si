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
  name: string,
): Promise<GeneratorResult> {
  logger.info(
    "Generating qualification function {name} for schema {schemaName}",
    {
      schemaName,
      name,
    },
  );
  logger.info("---");
  logger.info("");

  await generator.generateSchemaQualificationBase(project, schemaName);

  const paths = await generator.generateSchemaQualification(
    project,
    schemaName,
    name,
  );

  logger.info("");
  logger.info("---");
  logger.info(
    "Successfully generated qualification function for schema {schemaName}",
    {
      schemaName,
    },
  );
  logger.info(`  - ${name}`);

  ctx.analytics.trackEvent("schema_qualification_generate", {
    schemaName: schemaName,
    qualificationName: name,
  });

  return paths;
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}
