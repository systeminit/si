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
import { workspace$ } from "@/observable/workspace";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";
import { LabelList } from "@/api/sdf/dal/label_list";

export interface ListComponentNamesOnlyRequest extends Visibility {
  workspaceId: number;
}

export interface ListComponentNamesOnlyResponse {
  list: LabelList<number>;
}

const componentNamesOnlyList$ = combineLatest([
  standardVisibilityTriggers$,
]).pipe(
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
    const request: ListComponentNamesOnlyRequest = {
      ...visibility,
      workspaceId: workspace.id,
    };
    return sdf.get<ApiResponse<ListComponentNamesOnlyResponse>>(
      "component/list_components_names_only",
      request,
    );
  }),
  share(),
);

export function listComponentsNamesOnly(): Observable<
  ApiResponse<ListComponentNamesOnlyResponse>
> {
  return componentNamesOnlyList$;
}
