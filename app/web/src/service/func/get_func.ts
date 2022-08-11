import { ApiResponse } from "@/api/sdf";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";

export interface GetFuncArgs {
  id: number;
}

export type GetFuncResponse = Func;

const memo: {
  [key: string]: Observable<GetFuncResponse>;
} = {};

export const getFunc: (args: GetFuncArgs) => Observable<GetFuncResponse> =
  memoizedVisibilitySdfPipe(
    (visibility, sdf, args) =>
      sdf
        .get<ApiResponse<GetFuncResponse>>("func/get_func", {
          ...args,
          ...visibility,
        })
        .pipe(
          map((response) => {
            if (response.error) {
              GlobalErrorService.set(response);
              return nullFunc;
            }

            return response as GetFuncResponse;
          }),
        ),
    memo,
  );

export const nullFunc: GetFuncResponse = {
  id: 0,
  handler: undefined,
  kind: FuncBackendKind.Unset,
  name: "",
  code: undefined,
};
