import { ApiResponse, SDF } from "@/api/sdf";
import { Schema } from "@/api/sdf/dal/schema";
import { combineLatest, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";

export interface GetSchemaArgs {
  schemaId: number;
}

export interface GetSchemaRequest extends GetSchemaArgs, Visibility {}

export type GetSchemaResponse = Schema;

const getSchemaCollection: {
  [key: number]: Observable<ApiResponse<Schema>>;
} = {};

export function getSchema(
  args: GetSchemaArgs,
): Observable<ApiResponse<GetSchemaResponse>> {
  if (getSchemaCollection[args.schemaId]) {
    return getSchemaCollection[args.schemaId];
  }
  getSchemaCollection[args.schemaId] = combineLatest([
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.get<ApiResponse<GetSchemaResponse>>("schema/get_schema", {
        ...args,
        ...visibility,
      });
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getSchemaCollection[args.schemaId];
}
