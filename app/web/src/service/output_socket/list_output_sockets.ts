import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, from } from "rxjs";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { OutputSocket } from "@/api/sdf/dal/output_socket";
import { system$ } from "@/observable/system";

export interface ListOutputSocketArgs extends Visibility {
  schemaVariantId: number;
}

export interface ListOutputSocketRequest extends ListOutputSocketArgs {
  workspaceId?: number;
}

export interface ListOutputSocketResponse {
  inputSockets: OutputSocket[];
}

export function listOutputSockets(
  args: ListOutputSocketRequest,
): Observable<ApiResponse<ListOutputSocketResponse>> {
  return combineLatest([standardVisibilityTriggers$, system$, workspace$]).pipe(
    switchMap(([[visibility], system, workspace]) => {
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
      return sdf.get<ApiResponse<ListOutputSocketResponse>>(
        "output_socket/list_output_sockets",
        {
          ...args,
          ...visibility,
          systemId: system?.id,
          workspaceId: workspace.id,
        },
      );
    }),
  );
}
