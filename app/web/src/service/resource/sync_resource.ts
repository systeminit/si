import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { workspace$ } from "@/observable/workspace";
import { system$ } from "@/observable/system";
import { Resource, ResourceHealth } from "@/api/sdf/dal/resource";
import _ from "lodash";

export interface SyncResourceArgs {
  componentId: number;
}

export interface SyncResourceRequest extends SyncResourceArgs, Visibility {
  systemId?: number;
  workspaceId: number;
}

export type SyncResourceResponse = { resource: Resource };

export function syncResource(
  args: SyncResourceArgs,
): Observable<ApiResponse<SyncResourceResponse>> {
  return combineLatest([standardVisibilityTriggers$, system$, workspace$]).pipe(
    switchMap(([[_visibility], _system, workspace]) => {
      const bottle = Bottle.pop("default");
      const _sdf: SDF = bottle.container.SDF;
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
      return from([
        {
          resource: {
            id: args.componentId,
            timestamp: "0",
            error: "Boto Cor de Rosa Spotted",
            data: { "Saci-Pererê": { "Its just a prank bro": 3 } },
            health: ResourceHealth.Warning,
            entityType:
              "Eat Acarajé with Shrimps & Vatapa & Caruru & a lot of hot sauce",
          },
        },
      ]);
      //return sdf.get<ApiResponse<SyncResourceResponse>>(
      //  "resource/sync_resource",
      //  {
      //    ...args,
      //    ...visibility,
      //    systemId: system.id,
      //    workspaceId: workspace.id,
      //  },
      //);
    }),
  );
}
