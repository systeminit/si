import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { getLogger } from "../../../logger.ts";
import { Project } from "../../../project.ts";
import type { AbsoluteFilePath } from "../../../project.ts";

const logger = getLogger();

export async function callSchemaManagementGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
  name: string,
): Promise<GeneratorResult> {
  logger.info("Generating management function {name} for schema {schemaName}", {
    schemaName,
    name,
  });
  logger.info("---");
  logger.info("");

  await generator.generateSchemaManagementBase(project, schemaName);

  const paths = await generator.generateSchemaManagement(
    project,
    schemaName,
    name,
  );

  logger.info("");
  logger.info("---");
  logger.info(
    "Successfully generated management function for schema {schemaName}",
    {
      schemaName,
    },
  );
  logger.info(`  - ${name}`);

  ctx.analytics.trackEvent("schema_management_generate", {
    schemaName: schemaName,
    managementName: name,
  });

  return paths;
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}
