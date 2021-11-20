import { ApiResponse, SDF } from "@/api/sdf";
import { Schema } from "@/api/sdf/dal/schema";
import { combineLatest, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { schemaCreated$ } from "@/observable/schema";
import { switchMap } from "rxjs/operators";

export interface ListSchemaResponse {
  list: Array<Schema>;
}

const schemaList$ = combineLatest([
  standardVisibilityTriggers$,
  schemaCreated$,
]).pipe(
  switchMap(([[visibility]]) => {
    const bottle = Bottle.pop("default");
    const sdf: SDF = bottle.container.SDF;
    return sdf.get<ApiResponse<ListSchemaResponse>>(
      "schema/list_schemas",
      visibility,
    );
  }),
  shareReplay(1),
);

export function listSchemas(): Observable<ApiResponse<ListSchemaResponse>> {
  return schemaList$;
}
