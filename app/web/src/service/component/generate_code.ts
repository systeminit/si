import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { workspace$ } from "@/observable/workspace";
import { system$ } from "@/observable/system";
import _ from "lodash";

export interface GenerateCodeArgs {
  componentId: number;
}

export interface GenerateCodeRequest extends GenerateCodeArgs, Visibility {
  systemId?: number;
  workspaceId: number;
}

export type GenerateCodeResponse = { success: boolean };

export function generateCode(
  args: GenerateCodeArgs,
): Observable<ApiResponse<GenerateCodeResponse>> {
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
      return sdf.post<ApiResponse<GenerateCodeResponse>>(
        "component/generate_code",
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
