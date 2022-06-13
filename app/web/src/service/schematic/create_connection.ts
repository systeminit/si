import { Visibility } from "@/api/sdf/dal/visibility";
import { combineLatest, from, Observable, take, tap } from "rxjs";
import { ApiResponse, SDF } from "@/api/sdf";
import { visibility$ } from "@/observable/visibility";
import { workspace$ } from "@/observable/workspace";
import { switchMap } from "rxjs/operators";
import Bottle from "bottlejs";
import _ from "lodash";
import { editSessionWritten$ } from "@/observable/edit_session";

export interface CreateConnectionArgs {
  headNodeId: number;
  headSocketId: number;
  headInternalProviderId: number;
  tailNodeId: number;
  tailSocketId: number;
  tailExternalProviderId: number;
}

export interface CreateConnectionRequest
  extends CreateConnectionArgs,
    Visibility {
  workspaceId: number;
}

export function createConnection(
  args: CreateConnectionArgs,
): Observable<ApiResponse<void>> {
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
              message: "cannot make call without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      const request: CreateConnectionRequest = {
        ...args,
        ...visibility,
        workspaceId: workspace.id,
      };
      return sdf.post("schematic/create_connection", request).pipe(
        tap((response) => {
          if (!response.error) {
            editSessionWritten$.next(true);
          }
        }),
      );
    }),
  );
}
