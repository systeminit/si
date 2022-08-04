import { ApiResponse, SDF } from "@/api/sdf";
import Bottle from "bottlejs";
import { combineLatest, Observable } from "rxjs";
import { switchMap, share } from "rxjs/operators";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";

export interface GetFuncArgs {
  id: number;
}

export type GetFuncResponse = Func;

export function getFunc(
  args: GetFuncArgs,
): Observable<ApiResponse<GetFuncResponse>> {
  return combineLatest([standardVisibilityTriggers$]).pipe(
    switchMap(([[visibility]]) => {
      const sdf: SDF = Bottle.pop("default").container.SDF;
      return sdf.get<ApiResponse<GetFuncResponse>>("func/get_func", {
        ...args,
        ...visibility,
      });
    }),
    share(),
  );
}

export const nullFunc: GetFuncResponse = {
  id: 0,
  handler: undefined,
  kind: FuncBackendKind.Unset,
  name: "",
  code: undefined,
};
