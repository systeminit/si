import Bottle from "bottlejs";
import { combineLatest, Observable, take } from "rxjs";
import { switchMap } from "rxjs/operators";
import _ from "lodash";
import { ApiResponse, SDF } from "@/api/sdf";
import { DiagramKind } from "@/api/sdf/dal/diagram";
import { Visibility } from "@/api/sdf/dal/visibility";
import { NodePosition } from "@/api/sdf/dal/node_position";
import { visibility$ } from "@/observable/visibility";

export interface SetNodePositionArgs {
  diagramKind: DiagramKind;
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
      return sdf.post<ApiResponse<SetNodePositionResponse>>(
        "diagram/set_node_position",
        request,
      );
    }),
  );
}
