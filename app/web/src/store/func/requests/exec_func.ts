import { Visibility } from "@/api/sdf/dal/visibility";
import { ApiRequest } from "@/utils/pinia_api_tools";

export interface ExecFuncRequest {
  id: string;
}

export interface ExecFuncResponse {
  success: boolean;
}

export const execFunc = (
  params: ExecFuncRequest & Visibility,
  onSuccess?: (response: ExecFuncResponse) => void,
) =>
  new ApiRequest<ExecFuncResponse, typeof params>({
    method: "post",
    url: "func/exec_func",
    params,
    onSuccess,
  });
