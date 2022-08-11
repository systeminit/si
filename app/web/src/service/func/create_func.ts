import { ApiResponse } from "@/api/sdf";
import { combineLatest, Observable } from "rxjs";
import { map, switchMap } from "rxjs/operators";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import Bottle from "bottlejs";
import { SDF } from "@/api/sdf";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
export type CreateFuncResponse = Func;

export const nullFunc: CreateFuncResponse = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  code: undefined,
};
const memo: {
  [key: string]: Observable<CreateFuncResponse>;
} = {};

export const createFunc: () => Observable<CreateFuncResponse> = () =>
  combineLatest([standardVisibilityTriggers$]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.post<ApiResponse<CreateFuncResponse>>("func/create_func", {
        ...visibility,
      });
    }),
    map((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return nullFunc;
      }

      return response as CreateFuncResponse;
    }),
  );
