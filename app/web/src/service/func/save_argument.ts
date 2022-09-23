import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { FuncArgument } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";

export type SaveArgumentRequest = Omit<FuncArgument, "funcId">;

export interface SaveArgumentResponse {
  success: boolean;
}

export const saveArgument: (
  args: SaveArgumentRequest,
) => Promise<SaveArgumentResponse> = async (args) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const response = await firstValueFrom(
    sdf.post<ApiResponse<SaveArgumentResponse>>("func/save_argument", {
      ...args,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return { success: false };
  }

  return response as SaveArgumentResponse;
};
