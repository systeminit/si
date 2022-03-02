import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, from, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { CodeView } from "@/api/sdf/dal/code_view";

export interface GetCodeArgs {
  componentId: number;
}

export interface GetCodeRequest extends GetCodeArgs, Visibility {
  workspaceId: number;
}

export interface GetCodeResponse {
  codeViews: Array<CodeView>;
}

const getCodeCollection: {
  [key: number]: Observable<ApiResponse<GetCodeResponse>>;
} = {};

export function getCode(
  args: GetCodeArgs,
): Observable<ApiResponse<GetCodeResponse>> {
  if (getCodeCollection[args.componentId]) {
    return getCodeCollection[args.componentId];
  }
  getCodeCollection[args.componentId] = combineLatest([
    standardVisibilityTriggers$,
    workspace$,
  ]).pipe(
    switchMap(([[visibility], workspace]) => {
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
      return sdf.get<ApiResponse<GetCodeResponse>>("component/get_code", {
        ...args,
        ...visibility,
        workspaceId: workspace.id,
      });
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getCodeCollection[args.componentId];
}
