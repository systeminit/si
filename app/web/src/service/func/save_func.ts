import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";
import { Func } from "@/api/sdf/dal/func";

export interface SaveFuncRequest extends Func {
  schemaVariants: number[];
  components: number[];
}

export interface SaveFuncResponse {
  success: boolean;
  isRevertable: boolean;
}

export const saveFunc: (
  func: SaveFuncRequest,
) => Promise<SaveFuncResponse> = async (func) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const isDevMode = import.meta.env.DEV;
  const endpoint =
    isDevMode && func.isBuiltin ? "dev/save_func" : "func/save_func";

  const response = await firstValueFrom(
    sdf.post<ApiResponse<SaveFuncResponse>>(endpoint, {
      ...func,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return { success: false, isRevertable: false };
  }
  return response as SaveFuncResponse;
};
