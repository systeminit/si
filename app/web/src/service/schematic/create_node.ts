import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable, take } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { SchematicNode } from "@/api/sdf/dal/schematic";

export interface CreateNodeArgs {
  schemaId: number;
  x: number;
  y: number;
  systemId?: number;
  parentNodeId: number | null;
}

export interface CreateNodeRequest
  extends Omit<CreateNodeArgs, "x" | "y">,
    Visibility {
  workspaceId: number;
  x: string;
  y: string;
}

export interface CreateNodeResponse {
  node: SchematicNode;
}

export function createNode(
  args: CreateNodeArgs,
): Observable<ApiResponse<CreateNodeResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  return combineLatest([visibility$, workspace$]).pipe(
    take(1),
    switchMap(([visibility, workspace]) => {
      if (_.isNull(workspace)) {
        return from([
          {
            error: {
              statusCode: 10,
              message: "cannot create workspace without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      const request: CreateNodeRequest = {
        ...args,
        x: args.x.toString(),
        y: args.y.toString(),
        ...visibility,
        workspaceId: workspace.id,
      };
      return sdf.post<ApiResponse<CreateNodeResponse>>(
        "schematic/create_node",
        request,
      );
    }),
  );
}
