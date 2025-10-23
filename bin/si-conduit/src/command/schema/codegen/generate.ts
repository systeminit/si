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
  name: string,
) {
  logger.info("Generating codegen function {name} for schema {schemaName}", {
    schemaName,
    name,
  });
  logger.info("---");
  logger.info("");

  await generator.generateSchemaCodegenBase(project, schemaName);

  const paths = await generator.generateSchemaCodegen(
    project,
    schemaName,
    name,
  );

  logger.info("");
  logger.info("---");
  logger.info(
    "Successfully generated codegen function for schema {schemaName}",
    {
      schemaName,
    },
  );
  logger.info(`  - ${name}`);

  ctx.analytics.trackEvent("schema_codegen_generate", {
    schemaName: schemaName,
    codegenName: name,
  });

  return paths;
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}
