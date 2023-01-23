import { Func, FuncVariant } from "@/api/sdf/dal/func";
import { ApiRequest } from "@/store/lib/pinia_api_tools";
import { Visibility } from "@/api/sdf/dal/visibility";

export type ListedFuncView = Omit<Func, "code">;

export interface ListFuncsResponse {
  funcs: ListedFuncView[];
}

function nilId(): string {
  return "00000000000000000000000000";
}

export const nullListFunc: ListedFuncView = {
  id: nilId(),
  handler: "",
  variant: FuncVariant.Attribute,
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
