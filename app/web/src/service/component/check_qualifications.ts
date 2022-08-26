import { combineLatest, take, Observable } from "rxjs";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import _ from "lodash";
import { ApiResponse, SDF } from "@/api/sdf";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { Visibility } from "@/api/sdf/dal/visibility";
import { system$ } from "@/observable/system";

export interface CheckQualificationArgs {
  componentId: number;
}

export interface CheckQualificationRequest
  extends CheckQualificationArgs,
    Visibility {
  systemId?: number;
}

export type CheckQualificationResponse = { success: boolean };

export function checkQualifications(
  args: CheckQualificationArgs,
): Observable<ApiResponse<CheckQualificationResponse>> {
  return combineLatest([standardVisibilityTriggers$, system$]).pipe(
    take(1),
    switchMap(([[visibility], system]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.post<ApiResponse<CheckQualificationResponse>>(
        "component/check_qualifications",
        {
          ...args,
          ...visibility,
          systemId: system?.id,
        },
      );
    }),
  );
}
