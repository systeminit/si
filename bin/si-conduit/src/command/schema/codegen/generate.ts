import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import { getLogger } from "../../../logger.ts";
import { Project } from "../../../project.ts";
import type { AbsoluteFilePath } from "../../../project.ts";

const logger = getLogger();

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

  await generator.generateSchemaCodegenBase(project, schemaName);

  const paths = await generator.generateSchemaCodegen(
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
