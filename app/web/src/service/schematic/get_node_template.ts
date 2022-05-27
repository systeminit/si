import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable, shareReplay, map } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { SchematicNodeTemplate } from "@/api/sdf/dal/schematic";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";

// This datastructure should die after the backend starts returning SchematicNodeTemplate
interface NodeTemplate {
  kind: "component" | "deployment";
  label: {
    title: string;
    name: string;
  };
  schemaVariantId: number;
}

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
      return sdf.get<ApiResponse<NodeTemplate>>(
        "schematic/get_node_template",
        request,
      );
    }),
    map((raw) => {
      if (raw.error) return { error: raw.error };
      const template: GetNodeTemplateResponse = {
        name: raw.label.name,
        title: raw.label.title,
        kind: raw.kind,
        schemaVariantId: raw.schemaVariantId,
      };
      return template;
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getNodeTemplateCollection[key];
}
