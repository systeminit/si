import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { getLogger } from "../../../logger.ts";
import { Project } from "../../../project.ts";
import type { AbsoluteFilePath } from "../../../project.ts";

const logger = getLogger();

export async function callSchemaAuthGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
  name: string,
): Promise<GeneratorResult> {
  logger.info(
    "Generating authentication function {name} for schema {schemaName}",
    {
      schemaName,
      name,
    },
  );
  logger.info("---");
  logger.info("");

  await generator.generateSchemaAuthBase(project, schemaName);

  const paths = await generator.generateSchemaAuth(project, schemaName, name);

  logger.info("");
  logger.info("---");
  logger.info(
    "Successfully generated authentication function for schema {schemaName}",
    {
      schemaName,
    },
  );
  logger.info(`  - ${name}`);

  ctx.analytics.trackEvent("schema_authentication_generate", {
    schemaName: schemaName,
    authName: name,
  });

  return paths;
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}
