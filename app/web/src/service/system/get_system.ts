import Bottle from "bottlejs";
import _ from "lodash";
import { combineLatest, combineLatestWith, from, Observable, tap } from "rxjs";
import { switchMap } from "rxjs/operators";

import { ApiResponse, SDF } from "@/api/sdf";
import { System } from "../../api/sdf/dal/system";
import { Visibility } from "@/api/sdf/dal/visibility";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { workspace$ } from "@/observable/workspace";
import { SystemService } from "@/service/system";

export interface GetSystemArgs {
  systemId: number;
}

export interface GetSystemRequest extends GetSystemArgs, Visibility {
  workspaceId: number;
}

export interface GetSystemResponse {
  system: System;
}

export function getSystem(
  args: GetSystemArgs,
): Observable<ApiResponse<GetSystemResponse>> {
  return combineLatest([standardVisibilityTriggers$]).pipe(
    combineLatestWith(workspace$),
    switchMap(([[[visibility]], workspace]) => {
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
      const request: GetSystemRequest = {
        ...visibility,
        ...args,
        workspaceId: workspace.id,
      };
      return sdf
        .get<ApiResponse<GetSystemResponse>>("system/get_system", request)
        .pipe(
          tap((response) => {
            if (!response.error) {
              SystemService.switchTo(response.system);
            }
          }),
        );
    }),
  );
}
