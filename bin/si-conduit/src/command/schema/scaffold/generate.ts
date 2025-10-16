import { Project } from "../../../project.ts";

export async function call_schema_scaffold_generate(
  project: Project,
  schemaName: string,
) {
  console.log(
    project,
    schemaName,
    "Hello from schema scaffold generate",
  );
}
