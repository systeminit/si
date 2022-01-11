import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { Schema, SchemaKind } from "@/api/sdf/dal/schema";
import { combineLatest, from, Observable, take, tap } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { editSessionWritten$ } from "@/observable/edit_session";
import { Node } from "@/organisims/SchematicViewer/model/node";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";

// Note: eventually, this needs to include the name and the position. For now, just the ID is good enough.
export interface CreateNodeArgs {
  schemaId: number;
  rootNodeId: number;
  x: string;
  y: string;
}

export interface CreateNodeRequest extends CreateNodeArgs, Visibility {
  workspaceId: number;
}

export interface CreateNodeResponse {
  node: Node;
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
        ...visibility,
        workspaceId: workspace.id,
      };
      return sdf
        .post<ApiResponse<CreateNodeResponse>>("schematic/create_node", request)
        .pipe(
          tap((response) => {
            if (!response.error) {
              editSessionWritten$.next(true);
            }
          }),
        );
    }),
  );
}
