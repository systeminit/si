import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { getLogger } from "../../../logger.ts";
import { Project } from "../../../project.ts";
import type { AbsoluteFilePath } from "../../../project.ts";

const logger = getLogger();

export async function callSchemaActionGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
  name: string,
): Promise<GeneratorResult> {
  logger.info("Generating action function {name} for schema {schemaName}", {
    schemaName,
    name,
  });
  logger.info("---");
  logger.info("");

  await generator.generateSchemaActionBase(project, schemaName);

  const paths = await generator.generateSchemaAction(project, schemaName, name);

  logger.info("");
  logger.info("---");
  logger.info(
    "Successfully generated action function for schema {schemaName}",
    {
      schemaName,
    },
  );
  logger.info(`  - ${name}`);

  ctx.analytics.trackEvent("schema_action_generate", {
    schemaName: schemaName,
    actionName: name,
  });

  return paths;
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}
