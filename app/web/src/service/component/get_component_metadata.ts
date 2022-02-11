import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, combineLatestWith, from, Observable } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { workspace$ } from "@/observable/workspace";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";

export interface GetComponentMetadataArgs {
  componentId: number;
  systemId: number;
}

export interface GetComponentMetadataRequest
  extends GetComponentMetadataArgs,
    Visibility {
  workspaceId: number;
}

export interface GetComponentMetadataResponse {
  schemaName: string;
  qualified?: boolean;
  resourceHealth?: string;
}

export function getComponentMetadata(
  args: GetComponentMetadataArgs,
): Observable<ApiResponse<GetComponentMetadataResponse>> {
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
      const request: GetComponentMetadataRequest = {
        ...visibility,
        ...args,
        workspaceId: workspace.id,
      };
      return sdf.get<ApiResponse<GetComponentMetadataResponse>>(
        "component/get_component_metadata",
        request,
      );
    }),
  );
}
