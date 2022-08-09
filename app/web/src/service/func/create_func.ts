import { ApiResponse } from "@/api/sdf";
import { GlobalErrorService } from "@/service/global_error";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";

import { GetFuncResponse, nullFunc } from "./get_func";

export type CreateFuncResponse = GetFuncResponse;

const memo: {
  [key: string]: Observable<CreateFuncResponse>;
} = {};

export const createFunc: () => Observable<CreateFuncResponse> =
  memoizedVisibilitySdfPipe(
    (visibility, sdf) =>
      sdf
        .post<ApiResponse<CreateFuncResponse>>("func/create_func", {
          ...visibility,
        })
        .pipe(
          map((response) => {
            if (response.error) {
              GlobalErrorService.set(response);
              return nullFunc;
            }

            return response as CreateFuncResponse;
          }),
        ),
    memo,
  );
