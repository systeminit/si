import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, take, shareReplay } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { SchematicSchemaVariants } from "@/api/sdf/dal/schematic";
import _ from "lodash";

export type ListSchemaVariantsRequest = Visibility;
export type ListSchemaVariantsResponse = SchematicSchemaVariants;

export function listSchemaVariants(): Observable<
  ApiResponse<ListSchemaVariantsResponse>
> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  return combineLatest([standardVisibilityTriggers$]).pipe(
    take(1),
    switchMap(([[visibility]]) => {
      const request: ListSchemaVariantsRequest = {
        ...visibility,
      };
      return sdf.get<ApiResponse<ListSchemaVariantsResponse>>(
        "schematic/list_schema_variants",
        request,
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
}
