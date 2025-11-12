import { Context } from "../../../context.ts";
import * as generator from "../../../generators.ts";
import {
  functionKindToMaterializableEntity,
} from "../../../materialize.ts";
import { getLogger } from "../../../logger.ts";
import { FunctionKind, Project } from "../../../project.ts";
import type { AbsoluteFilePath } from "../../../project.ts";

const logger = getLogger();

export async function callSchemaFuncGenerate(
  ctx: Context,
  project: Project,
  schemaName: string,
  funcKind: FunctionKind,
  name: string,
  isOverlay: boolean,
): Promise<GeneratorResult> {
  const overlayMsg = isOverlay ? " overlay" : "";

  logger.info(
    `Generating${overlayMsg} ${funcKind} function {name} for schema {schemaName}`,
    {
      overlayMsg,
      schemaName,
      name,
    },
  );
  logger.info("---");
  logger.info("");

  const entity = functionKindToMaterializableEntity(funcKind);

  await generator.generateEntityBase(project, entity, schemaName, isOverlay);

  const paths = await generator.generateSchemaFunction(
    project,
    schemaName,
    name,
    entity,
    isOverlay,
  );

  logger.info("");
  logger.info("---");
  logger.info(
    `Successfully generated${overlayMsg} ${funcKind} function for schema {schemaName}`,
    {
      overlayMsg,
      schemaName,
    },
  );
  logger.info("  - {name}", { name });

  const eventName = `schema_${entity}_generate`;

  ctx.analytics.trackEvent(eventName, {
    schemaName: schemaName,
    actionName: name,
  });

  return paths;
}

export interface GeneratorResult {
  metadataPath: AbsoluteFilePath;
  codePath: AbsoluteFilePath;
}
