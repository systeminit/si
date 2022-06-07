import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { SchematicNodeTemplate } from "@/api/sdf/dal/schematic";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";

export interface GetNodeTemplateArgs {
  schemaId: number;
}

export interface GetNodeTemplateRequest
  extends GetNodeTemplateArgs,
    Visibility {
  workspaceId: number;
}

export type GetNodeTemplateResponse = SchematicNodeTemplate;

const getNodeTemplateCollection: {
  [key: string]: Observable<ApiResponse<GetNodeTemplateResponse>>;
} = {};

export function getNodeTemplate(
  args: GetNodeTemplateArgs,
): Observable<ApiResponse<GetNodeTemplateResponse>> {
  const key = args.schemaId;
  if (getNodeTemplateCollection[key]) {
    return getNodeTemplateCollection[key];
  }
  getNodeTemplateCollection[key] = combineLatest([
    standardVisibilityTriggers$,
    workspace$,
  ]).pipe(
    switchMap(([[visibility], workspace]) => {
      if (_.isNull(workspace)) {
        return from([
          {
            error: {
              statusCode: 10,
              message: "cannot get node template without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      const request: GetNodeTemplateRequest = {
        ...args,
        ...visibility,
        workspaceId: workspace.id,
      };
      return sdf.get<ApiResponse<SchematicNodeTemplate>>(
        "schematic/get_node_template",
        request,
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getNodeTemplateCollection[key];
}
