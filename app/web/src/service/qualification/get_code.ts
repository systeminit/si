import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";
import { QualificationPrototype } from "@/api/sdf/dal/qualification";

export interface GetCodeArgs {
  prototypeId: number;
}

export interface GetCodeRequest extends GetCodeArgs, Visibility {}

export interface GetCodeResponse {
  code: string;
  prototype: QualificationPrototype;
}

const getCodeCollection: {
  [key: number]: Observable<ApiResponse<GetCodeResponse>>;
} = {};

export function getCode(
  args: GetCodeArgs,
): Observable<ApiResponse<GetCodeResponse>> {
  if (getCodeCollection[args.prototypeId]) {
    return getCodeCollection[args.prototypeId];
  }
  getCodeCollection[args.prototypeId] = combineLatest([
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.get<ApiResponse<GetCodeResponse>>("qualification/get_code", {
        ...args,
        ...visibility,
      });
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getCodeCollection[args.prototypeId];
}
