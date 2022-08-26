import { combineLatest, from, Observable } from "rxjs";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import _ from "lodash";
import { ApiResponse, SDF } from "@/api/sdf";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { Visibility } from "@/api/sdf/dal/visibility";
import { workspace$ } from "@/observable/workspace";
import { system$ } from "@/observable/system";

export interface SyncResourceArgs {
  componentId: number;
}

export interface SyncResourceRequest extends SyncResourceArgs, Visibility {
  systemId?: number;
  workspaceId: number;
}

export type SyncResourceResponse = { success: boolean };

export function syncResource(
  args: SyncResourceArgs,
): Observable<ApiResponse<SyncResourceResponse>> {
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
      return sdf.post<ApiResponse<SyncResourceResponse>>(
        "component/sync_resource",
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
