import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { ApiResponse } from "@/api/sdf";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { GlobalErrorService } from "@/service/global_error";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";

export interface GetFuncArgs {
  id: number;
}

export interface QualificationAssocations {
  type: "qualification";
  schemaVariantIds: number[];
  componentIds: number[];
}

export interface AttributeAssociations {
  type: "attribute";
  props: {
    propId: number;
    name: string;
    componentId?: number;
    schemaVariantId: number;
  }[];
}

export type FuncAssociations = AttributeAssociations | QualificationAssocations;

export interface GetFuncResponse extends Func {
  isBuiltin: boolean;
  isRevertable: boolean;
  associations?: FuncAssociations;
}

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
    true,
  );

export const nullFunc: GetFuncResponse = {
  id: 0,
  handler: "",
  kind: FuncBackendKind.Unset,
  name: "",
  code: undefined,
  isBuiltin: false,
  isRevertable: false,
};
