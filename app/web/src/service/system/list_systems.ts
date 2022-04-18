import Bottle from "bottlejs";
import _ from "lodash";
import {
  combineLatest,
  combineLatestWith,
  from,
  Observable,
  share,
} from "rxjs";
import { switchMap } from "rxjs/operators";

import { ApiResponse, SDF } from "@/api/sdf";
import { LabelList } from "@/api/sdf/dal/label_list";
import { Visibility } from "@/api/sdf/dal/visibility";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { workspace$ } from "@/observable/workspace";

export interface ListSystemsRequest extends Visibility {
  workspaceId: number;
}

export interface ListSystemsResponse {
  list: LabelList<number>;
}

const systemsList$ = combineLatest([standardVisibilityTriggers$]).pipe(
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
    const request: ListSystemsRequest = {
      ...visibility,
      workspaceId: workspace.id,
    };
    return sdf.get<ApiResponse<ListSystemsResponse>>(
      "system/list_systems",
      request,
    );
  }),
  share(),
);

export function listSystems(): Observable<ApiResponse<ListSystemsResponse>> {
  return systemsList$;
}
