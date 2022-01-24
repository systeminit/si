import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { Qualification } from "@/api/sdf/dal/qualification";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";

export interface ListQualificationsArgs {
  componentId: number;
}

export interface ListQualificationsRequest
  extends ListQualificationsArgs,
    Visibility {
  workspaceId: number;
}

export type ListQualificationsResponse = Array<Qualification>;

const listQualificationsCollection: {
  [key: number]: Observable<ApiResponse<ListQualificationsResponse>>;
} = {};

export function listQualifications(
  args: ListQualificationsArgs,
): Observable<ApiResponse<ListQualificationsResponse>> {
  if (listQualificationsCollection[args.componentId]) {
    return listQualificationsCollection[args.componentId];
  }
  listQualificationsCollection[args.componentId] = combineLatest([
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
      return sdf.get<ApiResponse<ListQualificationsResponse>>(
        "component/list_qualifications",
        {
          ...args,
          ...visibility,
          workspaceId: workspace.id,
        },
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return listQualificationsCollection[args.componentId];
}
