import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { ApiResponse } from "@/api/sdf";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";
import { GlobalErrorService } from "@/service/global_error";

export interface ListedWorkflowHistoryView {
  id: number;
  title: string;
  description: string | null;
  status: "success" | "failure" | "running";
  created_at: string;
}

export type ListWorkflowsHistoryResponse = ListedWorkflowHistoryView[];

const memo: {
  [key: string]: Observable<ListWorkflowsHistoryResponse>;
} = {};

export const history: () => Observable<ListWorkflowsHistoryResponse> =
  memoizedVisibilitySdfPipe(
    (visibility, sdf) =>
      sdf
        .get<ApiResponse<ListWorkflowsHistoryResponse>>("workflow/history", {
          ...visibility,
        })
        .pipe(
          map((response) => {
            if (response.error) {
              GlobalErrorService.set(response);
              return [];
            }

            return response as ListWorkflowsHistoryResponse;
          }),
        ),
    memo,
  );
