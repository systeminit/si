import { firstValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { GlobalErrorService } from "@/service/global_error";
import { visibility$ } from "@/observable/visibility";

export interface WorkflowResolveRequest {
  id: number;
}

export enum WorkflowKind {
  Conditional = "conditional",
  Exceptional = "exceptional",
  Parallel = "parallel",
}

export interface WorkflowCommandStep {
  command: string;
  args: unknown;
}

export type WorkflowStep = WorkflowCommandStep | WorkflowTree;

export interface WorkflowTree {
  name: string;
  kind: WorkflowKind;
  steps: WorkflowStep[];
  args: unknown;
}

export interface WorkflowResolveResponse {
  json: string;
}

export const resolve: (
  arg: WorkflowResolveRequest,
) => Promise<WorkflowResolveResponse | null> = async (arg) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const response = await firstValueFrom(
    sdf.post<ApiResponse<WorkflowResolveResponse>>("workflow/resolve", {
      ...arg,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return null;
  }
  return response as WorkflowResolveResponse;
};
