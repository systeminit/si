import _ from "lodash";
import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable } from "rxjs";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { workspace$ } from "@/observable/workspace";
import { Visibility } from "@/api/sdf/dal/visibility";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { ResourceHealth } from "@/api/sdf/dal/resource";

export interface GetComponentsMetadataArgs {
  systemId: number;
}

export interface GetComponentsMetadataRequest
  extends GetComponentsMetadataArgs,
    Visibility {
  workspaceId: number;
}

export interface ComponentMetadata {
  componentId: number;
  schemaName: string;
  schemaLink: string | null;
  qualified: boolean | null;
  resourceHealth: ResourceHealth | null;
}
export interface GetComponentsMetadataResponse {
  data: ComponentMetadata[];
}

const getComponentsMetadataCollection: {
  [systemId: number]: Observable<ApiResponse<GetComponentsMetadataResponse>>;
} = {};

export function getComponentsMetadata(
  args: GetComponentsMetadataArgs,
): Observable<ApiResponse<GetComponentsMetadataResponse>> {
  if (getComponentsMetadataCollection[args.systemId]) {
    return getComponentsMetadataCollection[args.systemId];
  }
  getComponentsMetadataCollection[args.systemId] = combineLatest([
    workspace$,
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([workspace, [visibility]]) => {
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
      const request: GetComponentsMetadataRequest = {
        ...args,
        ...visibility,
        workspaceId: workspace.id,
      };
      return sdf.get<ApiResponse<GetComponentsMetadataResponse>>(
        "component/get_components_metadata",
        request,
      );
    }),
  );
  return getComponentsMetadataCollection[args.systemId];
}
