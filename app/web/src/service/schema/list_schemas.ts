import { combineLatest, Observable, share } from "rxjs";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { ApiResponse, SDF } from "@/api/sdf";
import { Schema } from "@/api/sdf/dal/schema";
import { standardVisibilityTriggers$ } from "@/observable/visibility";

export interface ListSchemaResponse {
  list: Array<Schema>;
}

const schemaList$ = combineLatest([standardVisibilityTriggers$]).pipe(
  switchMap(([[visibility]]) => {
    const bottle = Bottle.pop("default");
    const sdf: SDF = bottle.container.SDF;
    return sdf.get<ApiResponse<ListSchemaResponse>>(
      "schema/list_schemas",
      visibility,
    );
  }),
  share(),
);

export function listSchemas(): Observable<ApiResponse<ListSchemaResponse>> {
  return schemaList$;
}
