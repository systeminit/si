import Bottle from "bottlejs";
import _ from "lodash";
import { combineLatestWith, from, Observable, take, tap } from "rxjs";
import { switchMap } from "rxjs/operators";

import { ApiResponse, SDF } from "@/api/sdf";
import { System } from "@/api/sdf/dal/system";
import { Visibility } from "@/api/sdf/dal/visibility";
import { editSessionWritten$ } from "@/observable/edit_session";
import { visibility$ } from "@/observable/visibility";
import { workspace$ } from "@/observable/workspace";
import { SystemService } from "@/service/system";

export interface CreateSystemArgs {
  name: string;
}

export interface CreateSystemRequest extends CreateSystemArgs, Visibility {
  workspaceId: number;
}

export interface CreateSystemResponse {
  system: System;
}

export function createSystem(
  args: CreateSystemArgs,
): Observable<ApiResponse<CreateSystemResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return visibility$.pipe(
    take(1),
    combineLatestWith(workspace$),
    switchMap(([visibility, workspace]) => {
      if (_.isNull(workspace)) {
        return from([
          {
            error: {
              statusCode: 10,
              message: "cannot create a system without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      const request: CreateSystemRequest = {
        ...args,
        ...visibility,
        workspaceId: workspace.id,
      };
      return sdf
        .post<ApiResponse<CreateSystemResponse>>(
          "system/create_system",
          request,
        )
        .pipe(
          tap((response) => {
            if (!response.error) {
              SystemService.switchTo(response.system);
              editSessionWritten$.next(true);
            }
          }),
        );
    }),
  );
}
