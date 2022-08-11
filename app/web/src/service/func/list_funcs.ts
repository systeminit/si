import { ApiResponse } from "@/api/sdf";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";
import { Func } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";

export type ListedFuncView = Omit<Func, "code">;

export interface ListFuncsResponse {
  qualifications: ListedFuncView[];
}

const memo: {
  [key: string]: Observable<ListFuncsResponse>;
} = {};

export const listFuncs: () => Observable<ListFuncsResponse> =
  memoizedVisibilitySdfPipe(
    (visibility, sdf) =>
      sdf
        .get<ApiResponse<ListFuncsResponse>>("func/list_funcs", {
          ...visibility,
        })
        .pipe(
          map((response) => {
            if (response.error) {
              GlobalErrorService.set(response);
              return { qualifications: [] };
            }

            return response as ListFuncsResponse;
          }),
        ),
    memo,
  );
