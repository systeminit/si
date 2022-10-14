import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { FuncAssociations } from "@/service/func";
import { Visibility } from "@/api/sdf/dal/visibility";
import { ApiRequest } from "@/utils/pinia_api_tools";

export interface GetFuncArgs {
  id: number;
}

export interface GetFuncResponse extends Func {
  isBuiltin: boolean;
  isRevertible: boolean;
  associations?: FuncAssociations;
}

export const getFunc = (
  params: GetFuncArgs & Visibility,
  onSuccess: (response: GetFuncResponse) => void,
) =>
  new ApiRequest<GetFuncResponse, typeof params>({
    url: "func/get_func",
    params,
    onSuccess,
  });

export const nullFunc: GetFuncResponse = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  code: undefined,
  isBuiltin: false,
  isRevertible: false,
};
