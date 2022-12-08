import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { Visibility } from "@/api/sdf/dal/visibility";
import { ApiRequest } from "@/utils/pinia_api_tools";

export type CreateFuncResponse = Func;

export interface AttributeOptions {
  type: "attributeOptions";
  valueId: string;
  parentValueId?: string;
  componentId: string;
  schemaVariantId: string;
  schemaId: string;
  currentFuncId: string;
}

export interface CreateFuncRequest {
  kind: FuncBackendKind;
  options?: AttributeOptions;
}

function nilId(): string {
  return "00000000000000000000000000";
}

export const nullFunc: CreateFuncResponse = {
  id: nilId(),
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

export interface CreateBuiltinFuncRequest {
  name: string;
  kind: FuncBackendKind;
}

// might want to combine with above (like we do with saveFunc)?
export const createBuiltinFunc = (
  params: CreateBuiltinFuncRequest,
  onSuccess?: (response: CreateFuncResponse) => void,
) =>
  new ApiRequest<CreateFuncResponse, typeof params>({
    method: "post",
    url: "dev/create_func",
    params,
    onSuccess,
  });
