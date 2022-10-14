import { Func } from "@/api/sdf/dal/func";
import { Visibility } from "@/api/sdf/dal/visibility";
import { ApiRequest } from "@/utils/pinia_api_tools";
import { FuncAssociations } from "../types";

export interface SaveFuncRequest extends Func {
  associations?: FuncAssociations;
}

export interface SaveFuncResponse {
  associations?: FuncAssociations;
  success: boolean;
  isRevertible: boolean;
}

export const saveFunc = (
  params: SaveFuncRequest & Visibility,
  onSuccess: (response: SaveFuncResponse) => void,
) =>
  new ApiRequest<SaveFuncResponse, typeof params>({
    method: "post",
    url:
      import.meta.env.DEV && params.isBuiltin
        ? "dev/save_func"
        : "func/save_func",
    params,
    onSuccess,
  });
