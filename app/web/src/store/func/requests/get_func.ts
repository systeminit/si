import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { Visibility } from "@/api/sdf/dal/visibility";
import { ApiRequest } from "@/utils/pinia_api_tools";
import { FuncAssociations } from "../types";

export interface GetFuncArgs {
  id: string;
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

function nilId(): string {
  return "00000000000000000000000000";
}

export const nullFunc: GetFuncResponse = {
  id: nilId(),
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  code: undefined,
  isBuiltin: false,
  isRevertible: false,
};
