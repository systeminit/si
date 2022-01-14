import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatestWith, from, Observable, take, tap } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { editSessionWritten$ } from "@/observable/edit_session";
import { Component } from "@/api/sdf/dal/component";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";

export interface CreateApplicationArgs {
  name: string;
}

export interface CreateApplicationRequest
  extends CreateApplicationArgs,
    Visibility {
  workspaceId: number;
}

export interface CreateApplicationResponse {
  application: Component;
}

export function createApplication(
  args: CreateApplicationArgs,
): Observable<ApiResponse<CreateApplicationResponse>> {
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
              message: "cannot create an application without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      const request: CreateApplicationRequest = {
        ...args,
        ...visibility,
        workspaceId: workspace.id,
      };
      return sdf
        .post<ApiResponse<CreateApplicationResponse>>(
          "application/create_application",
          request,
        )
        .pipe(
          tap((response) => {
            if (!response.error) {
              editSessionWritten$.next(true);
            }
          }),
        );
    }),
  );
}
