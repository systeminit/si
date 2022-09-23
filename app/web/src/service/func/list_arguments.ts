import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { FuncArgument } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";

export interface ListArgumentsRequest {
  funcId: number;
}

export interface ListArgumentsResponse {
  arguments: FuncArgument[];
}

export const listArguments: (
  args: ListArgumentsRequest,
) => Promise<ListArgumentsResponse> = async (args) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const response = await firstValueFrom(
    sdf.get<ApiResponse<ListArgumentsResponse>>("func/list_arguments", {
      ...args,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return { arguments: [] };
  }

  return response as ListArgumentsResponse;
};
