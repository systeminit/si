import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { Project } from "../../../project.ts";
import type { AbsoluteFilePath } from "../../../project.ts";

export async function callSchemaCodegenGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
  codegenName: string,
) {
  const logger = ctx.logger;

  logger.info("Generating codegen function {codegenName} for {schemaName}", {
    schemaName,
    codegenName,
  });

  await generator.generateSchemaCodegenBase(ctx, project, schemaName);

  const paths = await generator.generateSchemaCodegen(
    ctx,
    project,
    schemaName,
    codegenName,
  );

  logger.info(
    "Successfully generated action function {codegenName} for schema {schemaName}",
    {
      schemaName,
      codegenName,
    },
  );

  ctx.analytics.trackEvent("generate_codegen_function", {
    schemaName: schemaName,
    codegenName: codegenName,
  });

  return paths;
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}
