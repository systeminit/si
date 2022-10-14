import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { ApiRequest } from "@/utils/pinia_api_tools";
import { Visibility } from "@/api/sdf/dal/visibility";

export type ListedFuncView = Omit<Func, "code">;

export interface ListFuncsResponse {
  funcs: ListedFuncView[];
}

export const nullListFunc: ListedFuncView = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  isBuiltin: false,
};

export const listFuncs = (
  visibility: Visibility,
  onSuccess: (response: ListFuncsResponse) => void,
) =>
  new ApiRequest<ListFuncsResponse, Visibility>({
    url: "func/list_funcs",
    params: {
      ...visibility,
    },
    onSuccess,
  });
