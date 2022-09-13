import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";

export interface ExecFuncRequest {
  id: number;
}

export interface ExecFuncResponse {
  success: boolean;
}

export const execFunc: (
  func: ExecFuncRequest,
) => Promise<ExecFuncResponse> = async (args) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const response = await firstValueFrom(
    sdf.post<ApiResponse<ExecFuncResponse>>("func/exec_func", {
      ...args,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return { success: false };
  }
  return response as ExecFuncResponse;
};
