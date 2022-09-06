import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";

export type CreateFuncResponse = Func;

export interface AttributeOptions {
  type: "attributeOptions";
  valueId: number;
  parentValueId?: number;
  componentId: number;
  schemaVariantId: number;
  schemaId: number;
  currentFuncId: number;
}

export interface CreateFuncRequest {
  kind: FuncBackendKind;
  options?: AttributeOptions;
}

export const nullFunc: CreateFuncResponse = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  code: undefined,
};

export const createFunc: (
  args: CreateFuncRequest,
) => Promise<CreateFuncResponse> = async (args) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const response = await firstValueFrom(
    sdf.post<ApiResponse<CreateFuncResponse>>("func/create_func", {
      ...args,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return nullFunc;
  }

  return response as CreateFuncResponse;
};
