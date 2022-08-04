import { ApiResponse, SDF } from "@/api/sdf";
import Bottle from "bottlejs";
import { combineLatest, Observable } from "rxjs";
import { switchMap, share } from "rxjs/operators";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { Visibility } from "@/api/sdf/dal/visibility";
import { Func } from "@/api/sdf/dal/func";

// These will become more complex interfaces as the feature is fleshed out
export type ListFuncsRequest = Visibility;
export type ListedFuncView = Omit<Func, "code">;

export interface ListFuncsResponse {
  qualifications: ListedFuncView[];
}

export function listFuncs(): Observable<ApiResponse<ListFuncsResponse>> {
  return combineLatest([standardVisibilityTriggers$]).pipe(
    switchMap(([[visibility]]) => {
      const sdf: SDF = Bottle.pop("default").container.SDF;
      return sdf.get<ApiResponse<ListFuncsResponse>>("func/list_funcs", {
        ...visibility,
      });
    }),
    share(),
  );
}
