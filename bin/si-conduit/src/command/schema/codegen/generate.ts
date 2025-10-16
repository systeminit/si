import { Project } from "../../../project.ts";

export async function call_schema_codegen_generate(
  project: Project,
  schemaName: string,
  codegenName: string,
) {
  console.log(
    project,
    schemaName,
    codegenName,
    "Hello from schema codegen generate",
  );
}
