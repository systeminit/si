import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { SchematicKind } from "@/api/sdf/dal/schematic";
import { combineLatest, from, Observable, take, tap } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { NodePosition } from "@/api/sdf/dal/node_position";
import { visibility$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { editSessionWritten$ } from "@/observable/edit_session";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";

export interface SetNodePositionArgs {
  deploymentNodeId?: number;
  schematicKind: SchematicKind;
  nodeId: number;
  rootNodeId: number;
  systemId?: number;
  x: string;
  y: string;
}

export interface SetNodePositionRequest
  extends SetNodePositionArgs,
    Visibility {
  workspaceId: number;
}

export interface SetNodePositionResponse {
  position: NodePosition;
}

export function setNodePosition(
  args: SetNodePositionArgs,
): Observable<ApiResponse<SetNodePositionResponse>> {
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
              message: "cannot set node position without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      const request: SetNodePositionRequest = {
        ...args,
        ...visibility,
        workspaceId: workspace.id,
      };
      return sdf
        .post<ApiResponse<SetNodePositionResponse>>(
          "schematic/set_node_position",
          request,
        )
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
