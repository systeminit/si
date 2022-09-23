import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { FuncArgument } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";

export type DeleteArgumentRequest = Pick<FuncArgument, "id">;

export interface DeleteArgumentResponse {
  success: boolean;
}

export const deleteArgument: (
  args: DeleteArgumentRequest,
) => Promise<DeleteArgumentResponse> = async (args) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const response = await firstValueFrom(
    sdf.post<ApiResponse<DeleteArgumentResponse>>("func/delete_argument", {
      ...args,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return { success: false };
  }

  return response as DeleteArgumentResponse;
};
