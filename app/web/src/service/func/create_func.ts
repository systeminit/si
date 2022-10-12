import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { Visibility } from "@/api/sdf/dal/visibility";
import { ApiRequest } from "@/utils/pinia_api_tools";

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
  isBuiltin: false,
};

export const createFunc = (
  params: CreateFuncRequest & Visibility,
  onSuccess?: (response: CreateFuncResponse) => void,
) =>
  new ApiRequest<CreateFuncResponse, typeof params>({
    method: "post",
    url: "func/create_func",
    params,
    onSuccess,
  });
