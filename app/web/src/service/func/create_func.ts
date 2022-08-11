import { ApiResponse } from "@/api/sdf";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";

export type CreateFuncResponse = Func;

export const nullFunc: CreateFuncResponse = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  code: undefined,
};
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



