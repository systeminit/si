export * from "./schema/create_schema";
import { createSchema } from "./schema/create_schema";
import { listSchemas } from "./schema/list_schemas";

export const SchemaService = {
  createSchema,
  listSchemas,
};
