import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";
import { WorkflowStatus } from "@/molecules/WorkflowStatusIcon.vue";

export interface WorkflowRunRequest {
  id: number;
  componentId: number | null;
}

export interface WorkflowRunnerState {
  created_at: string;
  id: number;
  pk: number;
  status: WorkflowStatus;
  updated_at: string;
  workflow_runner_id: number;
}

export interface WorkflowRunResponse {
  runId: number;
}

export const run: (
  arg: WorkflowRunRequest,
) => Promise<WorkflowRunResponse | null> = async (arg) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const response = await firstValueFrom(
    sdf.post<ApiResponse<WorkflowRunResponse>>("workflow/run", {
      id: arg.id,
      componentId: arg.componentId ?? -1,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return null;
  }
  return response as WorkflowRunResponse;
};
