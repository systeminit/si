import { ApiResponse, SDF } from "@/api/sdf";
import { take, Observable, combineLatest, switchMap, tap } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { editSessionWritten$ } from "@/observable/edit_session";
import Bottle from "bottlejs";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";

export interface CreateArgs {
  componentId: number;
  systemId?: number;
}

export interface CreateRequest extends CreateArgs, Visibility {}

export interface CreateResponse {
  prototypeId: number;
}

export function create(
  args: CreateArgs,
): Observable<ApiResponse<CreateResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  return combineLatest([standardVisibilityTriggers$]).pipe(
    take(1),
    switchMap(([[visibility]]) => {
      return sdf
        .post<ApiResponse<CreateResponse>>("qualification/create", {
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
