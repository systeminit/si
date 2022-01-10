import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, combineLatestWith, from, Observable, tap } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { Component } from "@/api/sdf/dal/component";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { application$, applicationNodeId$ } from "@/observable/application";

export interface GetApplicationArgs {
  applicationId: number;
}

export interface GetApplicationRequest extends GetApplicationArgs, Visibility {
  workspaceId: number;
}

export interface GetApplicationResponse {
  application: Component;
  applicationNodeId: number;
}

export function setCurrentApplication(
  args: GetApplicationArgs,
): Observable<ApiResponse<GetApplicationResponse>> {
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
      const request: GetApplicationRequest = {
        workspaceId: workspace.id,
        ...args,
        ...visibility,
      };
      return sdf.get<ApiResponse<GetApplicationResponse>>(
        "application/get_application",
        request,
      );
    }),
    tap((response) => {
      if (!response.error) {
        application$.next(response.application);
        applicationNodeId$.next(response.applicationNodeId);
      }
    }),
  );
}
