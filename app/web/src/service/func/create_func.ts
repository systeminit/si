import { ApiResponse } from "@/api/sdf";
import { firstValueFrom } from "rxjs";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import Bottle from "bottlejs";
import { SDF } from "@/api/sdf";
import { visibility$ } from "@/observable/visibility";
export type CreateFuncResponse = Func;

export const nullFunc: CreateFuncResponse = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  code: undefined,
};

export const createFunc: () => Promise<CreateFuncResponse> = async () => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const response = await firstValueFrom(
    sdf.post<ApiResponse<CreateFuncResponse>>("func/create_func", {
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return nullFunc;
  }

  return response as CreateFuncResponse;
};
