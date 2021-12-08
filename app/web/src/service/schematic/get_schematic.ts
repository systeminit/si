import { ApiResponse, SDF } from "@/api/sdf";
import { Schematic } from "@/api/sdf/dal/schematic";
import { combineLatest, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";

export interface GetSchematicArgs {
  context: string;
}

export interface GetSchematicRequest extends GetSchematicArgs, Visibility {}

export type GetSchematicResponse = Schematic;

const getSchematicCollection: {
  [key: string]: Observable<ApiResponse<Schematic>>;
} = {};

export function getSchematic(
  args: GetSchematicArgs,
): Observable<ApiResponse<GetSchematicResponse>> {
  if (getSchematicCollection[args.context]) {
    return getSchematicCollection[args.context];
  }
  getSchematicCollection[args.context] = combineLatest([
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.get<ApiResponse<GetSchematicResponse>>(
        "schematic/get_schematic",
        {
          ...args,
          ...visibility,
        },
      );
    }),
    shareReplay(1),
  );
  return getSchematicCollection[args.context];
}
