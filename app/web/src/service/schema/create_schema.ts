import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { Schema, SchemaKind } from "@/api/sdf/dal/schema";
import { Observable, take, tap } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { editSessionWritten$ } from "@/observable/edit_session";

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
      return sdf
        .post<ApiResponse<CreateSchemaResponse>>("schema/create_schema", {
          ...args,
          ...visibility,
        })
        .pipe(
          tap((response) => {
            if (!response.error) {
              editSessionWritten$.next(true);
            }
          }),
        );
    }),
  );
}
