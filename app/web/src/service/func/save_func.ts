import { ApiResponse } from "@/api/sdf";
import { combineLatest, Observable } from "rxjs";
import { map, switchMap, take } from "rxjs/operators";
import { GlobalErrorService } from "@/service/global_error";
import Bottle from "bottlejs";
import { SDF } from "@/api/sdf";
import { visibility$ } from "@/observable/visibility";
import { Func } from "@/api/sdf/dal/func";

export type SaveFuncRequest = Func;

export interface SaveFuncResponse {
  success: boolean;
}

export const saveFunc: (
  func: SaveFuncRequest,
) => Observable<SaveFuncResponse> = (func) =>
  combineLatest([visibility$]).pipe(
    take(1),
    switchMap(([visibility]) => {
      console.log("saving..");
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.post<ApiResponse<SaveFuncResponse>>("func/save_func", {
        ...func,
        ...visibility,
      });
    }),
    map((response) => {
      if (response.error) {
        GlobalErrorService.set(response);
        return { success: false };
      }

      return response as SaveFuncResponse;
    }),
  );
