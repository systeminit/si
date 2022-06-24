import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { SchematicKind } from "@/api/sdf/dal/schematic";
import { combineLatest, Observable, take, tap } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { NodePosition } from "@/api/sdf/dal/node_position";
import { visibility$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { editSessionWritten$ } from "@/observable/edit_session";
import _ from "lodash";

export interface SetNodePositionArgs {
  deploymentNodeId: number | null;
  schematicKind: SchematicKind;
  nodeId: number;
  systemId?: number;
  x: string;
  y: string;
}

export interface SetNodePositionRequest
  extends SetNodePositionArgs,
    Visibility {}

export interface SetNodePositionResponse {
  position: NodePosition;
}

export function setNodePosition(
  args: SetNodePositionArgs,
): Observable<ApiResponse<SetNodePositionResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  return combineLatest([visibility$]).pipe(
    take(1),
    switchMap(([visibility]) => {
      const request: SetNodePositionRequest = {
        ...args,
        ...visibility,
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
