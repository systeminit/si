import { ApiResponse } from "@/api/sdf";
import { memoizedVisibilitySdfPipe } from "@/utils/memoizedVisibilitySdfPipes";
import { Func } from "@/api/sdf/dal/func";

export type ListedFuncView = Omit<Func, "code">;

export interface ListFuncsResponse {
  qualifications: ListedFuncView[];
}

export const listFuncs = memoizedVisibilitySdfPipe((visibility, sdf) =>
  sdf.get<ApiResponse<ListFuncsResponse>>("func/list_funcs", { ...visibility }),
);
