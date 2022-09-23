import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { FuncArgument, FuncArgumentKind } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";

export type CreateArgumentResponse = FuncArgument;
export type CreateArgumentRequest = Omit<FuncArgument, "id">;

export const nullArgument: FuncArgument = {
  id: 0,
  funcId: 0,
  name: "",
  kind: FuncArgumentKind.Boolean,
  elementKind: undefined,
};

export const createArgument: (
  args: CreateArgumentRequest,
) => Promise<CreateArgumentResponse> = async (args) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const response = await firstValueFrom(
    sdf.post<ApiResponse<CreateArgumentResponse>>("func/create_argument", {
      ...args,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return nullArgument;
  }

  return response as CreateArgumentResponse;
};
