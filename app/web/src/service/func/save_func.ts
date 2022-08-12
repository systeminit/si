import { ApiResponse } from "@/api/sdf";
import { firstValueFrom } from "rxjs";
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
) => Promise<SaveFuncResponse> = async (func) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const response = await firstValueFrom(
    sdf.post<ApiResponse<SaveFuncResponse>>("func/save_func", {
      ...func,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return { success: false };
  }
  return response as SaveFuncResponse;
};
