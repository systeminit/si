import { Project } from "../../../project.ts";

export async function call_schema_action_generate(
  project: Project,
  schemaName: string,
  actionName: string,
) {
  console.log(
    project,
    schemaName,
    actionName,
    "Hello from schema action generate",
  );

  console.log(
    project.relativeActionFuncCodePath(schemaName, actionName),
    project.actionFuncCodePath(schemaName, actionName),
    "going to create code",
  );
  console.log(
    project.relativeActionFuncMetadataPath(schemaName, actionName),
    project.actionFuncMetadataPath(schemaName, actionName),
    "going to create metadata",
  );
}
