export * from "./schema/create_schema";
import { createSchema } from "./schema/create_schema";
import { listSchemas } from "./schema/list_schemas";
import { getSchema } from "./schema/get_schema";

export const SchemaService = {
  createSchema,
  listSchemas,
  getSchema,
};
