import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { workspace$ } from "@/observable/workspace";
import { system$ } from "@/observable/system";
import { Resource } from "@/api/sdf/dal/resource";
import _ from "lodash";

export interface GetResourceArgs {
  componentId: number;
}

export interface GetResourceRequest extends GetResourceArgs, Visibility {
  systemId?: number;
  workspaceId: number;
}

export type GetResourceResponse = { resource: Resource };

export function getResource(
  args: GetResourceArgs,
): Observable<ApiResponse<GetResourceResponse>> {
  return combineLatest([standardVisibilityTriggers$, system$, workspace$]).pipe(
    switchMap(([[visibility], system, workspace]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      if (_.isNull(workspace)) {
        return from([
          {
            error: {
              statusCode: 10,
              message: "cannot make call without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      return sdf.get<ApiResponse<GetResourceResponse>>(
        "component/get_resource",
        {
          ...args,
          ...visibility,
          systemId: system?.id,
          workspaceId: workspace.id,
        },
      );
    }),
  );
}
