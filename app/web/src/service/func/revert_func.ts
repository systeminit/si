import { Visibility } from "@/api/sdf/dal/visibility";
import { ApiRequest } from "@/utils/pinia_api_tools";

export interface RevertFuncRequest {
  id: number;
}

export interface RevertFuncResponse {
  success: boolean;
}

export const revertFunc = (
  params: RevertFuncRequest & Visibility,
  onSuccess: (response: RevertFuncResponse) => void,
) =>
  new ApiRequest<RevertFuncResponse, typeof params>({
    method: "post",
    url: "func/revert_func",
    params,
    onSuccess,
  });
