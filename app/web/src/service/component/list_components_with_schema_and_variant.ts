import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, combineLatestWith, from, Observable } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { shareReplay, switchMap } from "rxjs/operators";
import { workspace$ } from "@/observable/workspace";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";
import { LabelList } from "@/api/sdf/dal/label_list";
import { ComponentWithSchemaAndVariant } from "@/api/sdf/dal/component";

export interface ListComponentsWithSchemaAndVariantRequest extends Visibility {
  workspaceId: number;
}

export interface ListComponentsWithSchemaAndVariantResponse {
  list: LabelList<ComponentWithSchemaAndVariant>;
}

const componentWithSchemaAndVariantList$ = combineLatest([
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
    const request: ListComponentsWithSchemaAndVariantRequest = {
      ...visibility,
      workspaceId: workspace.id,
    };
    return sdf.get<ApiResponse<ListComponentsWithSchemaAndVariantResponse>>(
      "component/list_components_with_schema_and_variant",
      request,
    );
  }),
  shareReplay({ bufferSize: 1, refCount: true }),
);

export function listComponentsWithSchemaAndVariant(): Observable<
  ApiResponse<ListComponentsWithSchemaAndVariantResponse>
> {
  return componentWithSchemaAndVariantList$;
}
