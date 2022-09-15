import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { ApiResponse } from "@/api/sdf";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";
import { GlobalErrorService } from "@/service/global_error";

export interface ListedWorkflowView {
  id: number;
  title: string;
  description: string | null;
  link: string | null;
  components: Array<{
    id: number;
    name: string;
  }>;
  schemaName: string | null;
  schemaVariantName: string | null;
}

export type ListWorkflowsResponse = ListedWorkflowView[];

const memo: {
  [key: string]: Observable<ListWorkflowsResponse>;
} = {};

export const list: () => Observable<ListWorkflowsResponse> =
  memoizedVisibilitySdfPipe(
    (visibility, sdf) =>
      sdf
        .get<ApiResponse<ListWorkflowsResponse>>("workflow/list", {
          ...visibility,
        })
        .pipe(
          map((response) => {
            if (response.error) {
              GlobalErrorService.set(response);
              return [];
            }

            return response as ListWorkflowsResponse;
          }),
        ),
    memo,
  );
