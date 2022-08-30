import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { ApiResponse } from "@/api/sdf";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";

export interface ListedFuncView extends Omit<Func, "code"> {
  isBuiltin: boolean;
}

export interface ListFuncsResponse {
  funcs: ListedFuncView[];
}

export const nullListFunc: ListedFuncView = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  isBuiltin: false,
};

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
              return {} as ListFuncsResponse;
            }

            return response as ListFuncsResponse;
          }),
        ),
    memo,
  );
