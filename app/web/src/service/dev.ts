import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { visibility$ } from "@/observable/visibility";
import { ApiResponse, SDF } from "@/api/sdf";
import { GlobalErrorService } from "@/service/global_error";
import { FuncBackendKind } from "@/api/sdf/dal/func";
import { CreateFuncResponse, nullFunc } from "@/service/func/create_func";
import { SaveFuncRequest } from "@/service/func/save_func";

export async function createBuiltinFunc(data: {
  name: string;
  kind: FuncBackendKind;
}): Promise<CreateFuncResponse> {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const response = await firstValueFrom(
    sdf.post<ApiResponse<CreateFuncResponse>>("dev/create_func", {
      ...data,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return nullFunc;
  }
  return response as CreateFuncResponse;
}

export interface SaveFuncResponse {
  success: boolean;
}

export const saveBuiltinFunc: (
  func: SaveFuncRequest,
) => Promise<SaveFuncResponse> = async (func) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const response = await firstValueFrom(
    sdf.post<ApiResponse<SaveFuncResponse>>("dev/save_func", {
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

export const DevService = {
  createBuiltinFunc,
  saveBuiltinFunc,
};
