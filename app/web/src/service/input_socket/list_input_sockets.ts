import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, from } from "rxjs";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { InputSocket } from "@/api/sdf/dal/input_socket";
import { system$ } from "@/observable/system";

export interface ListInputSocketArgs extends Visibility {
  schemaVariantId: number;
}

export interface ListInputSocketRequest extends ListInputSocketArgs {
  workspaceId?: number;
}

export interface ListInputSocketResponse {
  inputSockets: InputSocket[];
}

export function listInputSockets(
  args: ListInputSocketRequest,
): Observable<ApiResponse<ListInputSocketResponse>> {
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
      return sdf.get<ApiResponse<ListInputSocketResponse>>(
        "input_socket/list_input_sockets",
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
