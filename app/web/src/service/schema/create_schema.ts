import Bottle from "bottlejs";
import { Observable, take } from "rxjs";
import { switchMap } from "rxjs/operators";
import { ApiResponse, SDF } from "@/api/sdf";
import { Schema, SchemaKind } from "@/api/sdf/dal/schema";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";

export interface CreateSchemaArgs {
  name: string;
  kind: SchemaKind;
}

export interface CreateSchemaRequest extends CreateSchemaArgs, Visibility {}

export interface CreateSchemaResponse {
  schema: Schema;
}

export function createSchema(
  args: CreateSchemaArgs,
): Observable<ApiResponse<CreateSchemaResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return visibility$.pipe(
    take(1),
    switchMap((visibility) => {
      return sdf.post<ApiResponse<CreateSchemaResponse>>(
        "schema/create_schema",
        {
          ...args,
          ...visibility,
        },
      );
    }),
  );
}
