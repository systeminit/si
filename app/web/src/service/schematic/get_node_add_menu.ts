import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { MenuFilter, MenuItem } from "@/api/sdf/dal/schematic";

export interface GetNodeAddMenuArgs {
  menuFilter: MenuFilter;
}

export interface GetNodeAddMenuRequest extends GetNodeAddMenuArgs, Visibility {}

export type GetNodeAddMenuResponse = MenuItem[];

const getNodeAddMenuCollection: {
  [key: string]: Observable<ApiResponse<GetNodeAddMenuResponse>>;
} = {};

export function getNodeAddMenu(
  args: GetNodeAddMenuArgs,
): Observable<ApiResponse<GetNodeAddMenuResponse>> {
  const key = `${args.menuFilter.schematicKind}:${args.menuFilter.rootComponentId}`;
  if (getNodeAddMenuCollection[key]) {
    return getNodeAddMenuCollection[key];
  }
  getNodeAddMenuCollection[key] = combineLatest([
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.post<ApiResponse<GetNodeAddMenuResponse>>(
        "schematic/get_node_add_menu",
        {
          ...args,
          ...visibility,
        },
      );
    }),
    shareReplay(1),
  );
  return getNodeAddMenuCollection[key];
}
