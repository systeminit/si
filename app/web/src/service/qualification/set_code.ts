import { ApiResponse, SDF } from "@/api/sdf";
import { take, Observable, combineLatest, switchMap, tap } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { editSessionWritten$ } from "@/observable/edit_session";
import Bottle from "bottlejs";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";

export interface SetCodeArgs {
  prototypeId: number;
  prototypeTitle: string;
  code: string;
  systemId?: number;
}

export interface SetCodeRequest extends SetCodeArgs, Visibility {}

export interface SetCodeResponse {
  success: boolean;
}

export function setCode(
  args: SetCodeArgs,
): Observable<ApiResponse<SetCodeResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  return combineLatest([standardVisibilityTriggers$]).pipe(
    take(1),
    switchMap(([[visibility]]) => {
      return sdf
        .post<ApiResponse<SetCodeResponse>>("qualification/set_code", {
          ...args,
          ...visibility,
        })
        .pipe(
          tap((response) => {
            if (!response.error) {
              editSessionWritten$.next(true);
            }
          }),
        );
    }),
  );
}
