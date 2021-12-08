import { ApiResponse, SDF } from "@/api/sdf";
import {
  combineLatest,
  combineLatestWith,
  from,
  Observable,
  share,
} from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Component } from "@/api/sdf/dal/component";
import { workspace$ } from "@/observable/workspace";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";

export interface ListApplicationRequest extends Visibility {
  workspaceId: number;
}

export interface ListApplicationItem {
  application: Component;

  // These populated the former card display
  //servicesWithResources: [];
  //systems: [];
  //changeSetCounts: {
  //  open: number;
  //  closed: number;
  //};
}

export interface ListApplicationResponse {
  list: Array<ListApplicationItem>;
}

const applicationList$ = combineLatest([standardVisibilityTriggers$]).pipe(
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
    const request: ListApplicationRequest = {
      ...visibility,
      workspaceId: workspace.id,
    };
    return sdf.get<ApiResponse<ListApplicationResponse>>(
      "application/list_applications",
      request,
    );
  }),
  share(),
);

export function listApplications(): Observable<
  ApiResponse<ListApplicationResponse>
> {
  return applicationList$;
}
