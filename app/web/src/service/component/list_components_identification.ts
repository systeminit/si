import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, combineLatestWith, from, Observable } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { shareReplay, switchMap } from "rxjs/operators";
import { workspace$ } from "@/observable/workspace";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";
import { LabelList } from "@/api/sdf/dal/label_list";
import { ComponentIdentification } from "@/api/sdf/dal/component";

export interface ListComponentsIdentificationRequest extends Visibility {
  workspaceId: number;
}

export interface ListComponentsIdentificationResponse {
  list: LabelList<ComponentIdentification>;
}

const componentIdentificationList$ = combineLatest([
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
    const request: ListComponentsIdentificationRequest = {
      ...visibility,
      workspaceId: workspace.id,
    };
    return sdf.get<ApiResponse<ListComponentsIdentificationResponse>>(
      "component/list_components_identification",
      request,
    );
  }),
  shareReplay({ bufferSize: 1, refCount: true }),
);

export function listComponentsIdentification(): Observable<
  ApiResponse<ListComponentsIdentificationResponse>
> {
  return componentIdentificationList$;
}
