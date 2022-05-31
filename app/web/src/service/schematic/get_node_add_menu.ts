import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { MenuFilter, MenuItem } from "@/api/sdf/dal/menu";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";

export interface GetNodeAddMenuArgs {
  menuFilter: MenuFilter;
}

export interface GetNodeAddMenuRequest extends GetNodeAddMenuArgs, Visibility {
  workspaceId: number;
}

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
    workspace$,
  ]).pipe(
    switchMap(([[visibility], workspace]) => {
      if (_.isNull(workspace)) {
        return from([
          {
            error: {
              statusCode: 10,
              message: "cannot get node menu without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      const request: GetNodeAddMenuRequest = {
        ...args,
        ...visibility,
        workspaceId: workspace.id,
      };
      return sdf.post<ApiResponse<GetNodeAddMenuResponse>>(
        "schematic/get_node_add_menu",
        request,
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getNodeAddMenuCollection[key];
}
