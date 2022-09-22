import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { Resource } from "@/api/sdf/dal/resource";
import { ApiResponse } from "@/api/sdf";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";
import { GlobalErrorService } from "@/service/global_error";

export interface WorkflowRunInfo {
  id: number;
  title: string;
  description: string | null;
  status: "success" | "failure" | "running";
  createdResources: Resource[];
  updatedResources: Resource[];
  created_at: string;
  logs: string[];
}

export interface WorkflowRunInfoRequest {
  id: number;
}

const memo: {
  [key: string]: Observable<WorkflowRunInfo | null>;
} = {};

export const info: (
  arg: WorkflowRunInfoRequest,
) => Observable<WorkflowRunInfo | null> = memoizedVisibilitySdfPipe(
  (visibility, sdf, arg) =>
    sdf
      .get<ApiResponse<WorkflowRunInfo>>("workflow/info", {
        ...arg,
        ...visibility,
      })
      .pipe(
        map((response) => {
          if (response.error) {
            GlobalErrorService.set(response);
            return null;
          }

          return response as WorkflowRunInfo;
        }),
      ),
  memo,
);
