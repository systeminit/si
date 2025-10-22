import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { Project } from "../../../project.ts";
import type { AbsoluteFilePath } from "../../../project.ts";

export async function callSchemaActionGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
  actionName: string,
): Promise<GeneratorResult> {
  const logger = ctx.logger;

  logger.info("Generating action function {actionName} for {schemaName}", {
    schemaName,
    actionName,
  });

  await generator.generateSchemaActionBase(ctx, project, schemaName);

  const paths = await generator.generateSchemaAction(
    ctx,
    project,
    schemaName,
    actionName,
  );

  logger.info(
    "Successfully generated action function {actionName} for schema {schemaName}",
    {
      schemaName,
      actionName,
    },
  );

  ctx.analytics.trackEvent("generate_action_function", {
    schemaName: schemaName,
    actionName: actionName,
  });

  return paths;
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}
