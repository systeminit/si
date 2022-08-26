import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";

export interface WorkflowRunRequest {
  id: number;
}

export interface WorkflowRunResponse {
  logs: string[];
}

export const run: (
  arg: WorkflowRunRequest,
) => Promise<WorkflowRunResponse | null> = async (arg) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const response = await firstValueFrom(
    sdf.post<ApiResponse<WorkflowRunResponse>>("workflow/run", {
      ...arg,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return null;
  }
  return response as WorkflowRunResponse;
};
