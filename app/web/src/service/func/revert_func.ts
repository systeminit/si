import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";

export interface RevertFuncRequest {
  id: number;
}

export interface RevertFuncResponse {
  success: boolean;
}

export const revertFunc: (
  func: RevertFuncRequest,
) => Promise<RevertFuncResponse> = async (args) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const response = await firstValueFrom(
    sdf.post<ApiResponse<RevertFuncResponse>>("func/revert_func", {
      ...args,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return { success: false };
  }
  return response as RevertFuncResponse;
};
