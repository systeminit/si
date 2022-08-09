import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { ComponentDiff } from "@/api/sdf/dal/component";

export interface GetDiffArgs {
  componentId: number;
}

export interface GetDiffResponse {
  componentDiff: ComponentDiff;
}

const getDiffCollection: {
  [key: number]: Observable<ApiResponse<GetDiffResponse>>;
} = {};

export function getDiff(
  args: GetDiffArgs,
): Observable<ApiResponse<GetDiffResponse>> {
  if (getDiffCollection[args.componentId]) {
    return getDiffCollection[args.componentId];
  }
  getDiffCollection[args.componentId] = combineLatest([
    standardVisibilityTriggers$,
    workspace$,
  ]).pipe(
    switchMap(([[visibility], workspace]) => {
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
      return sdf.get<ApiResponse<GetDiffResponse>>("component/get_diff", {
        ...args,
        ...visibility,
      });
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getDiffCollection[args.componentId];
}
