import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { Schema, SchemaKind } from "@/api/sdf/dal/schema";
import { schemaListRefresh$ } from "@/observable/schema";

interface CreateSchemaRequest {
  name: String;
  kind: SchemaKind;
}

interface CreateSchemaResponse {
  schema: Schema;
}

export async function createSchema(
  request: CreateSchemaRequest,
): Promise<ApiResponse<CreateSchemaResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const result: ApiResponse<CreateSchemaResponse> = await sdf.post(
    "schema/create_schema",
    request,
  );
  if (result.error) {
    return result;
  }
  schemaListRefresh$.next(true);
  return result;
}
