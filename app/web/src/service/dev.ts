import {
  combineLatest,
  shareReplay,
  map,
  switchMap,
  firstValueFrom,
} from "rxjs";
import { useObservable } from "@vueuse/rxjs";
import Bottle from "bottlejs";
import { visibility$ } from "@/observable/visibility";
import { ApiResponse, sdf, SDF } from "@/api/sdf";
import { GlobalErrorService } from "@/service/global_error";
import { system$ } from "@/observable/system";
import { Func, FuncBackendKind } from "@/api/sdf/dal/func";
import { CreateFuncResponse, nullFunc } from "@/service/func/create_func";

/**
 * Gets the (shorthand) current Git sha of HEAD. This could be done in an npm script with an
 * exported variable, but this SDF call is also used to dogfood the "dev" endpoint.
 *
 * Returns "null" if not in "dev" mode.
 */
function useCurrentGitSha() {
  if (import.meta.env.DEV) {
    interface GetCurrentGitShaResponse {
      sha: string;
    }

    const currentGitSha$ = combineLatest([system$, visibility$]).pipe(
      switchMap(([system, visibility]) => {
        return sdf.get<ApiResponse<GetCurrentGitShaResponse>>(
          "dev/get_current_git_sha",
          {
            ...(system?.id && { systemId: system.id }),
            ...visibility,
          },
        );
      }),
      map((apiResponse) => {
        if (apiResponse.error) {
          GlobalErrorService.set(apiResponse);
          return null;
        }
        return apiResponse;
      }),
      shareReplay({ bufferSize: 1, refCount: true }),
    );

    return useObservable(currentGitSha$);
  }
  return null;
}

export async function createBuiltinFunc(data: {
  name: string;
  kind: FuncBackendKind;
}): Promise<CreateFuncResponse> {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const response = await firstValueFrom(
    sdf.post<ApiResponse<CreateFuncResponse>>("dev/create_func", {
      ...data,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return nullFunc;
  }
  return response as CreateFuncResponse;
}

export interface SaveFuncRequest extends Func {
  schemaVariants: number[];
  components: number[];
}

export interface SaveFuncResponse {
  success: boolean;
}

export const saveBuiltinFunc: (
  func: SaveFuncRequest,
) => Promise<SaveFuncResponse> = async (func) => {
  const visibility = await firstValueFrom(visibility$);
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;

  const response = await firstValueFrom(
    sdf.post<ApiResponse<SaveFuncResponse>>("dev/save_func", {
      ...func,
      ...visibility,
    }),
  );

  if (response.error) {
    GlobalErrorService.set(response);
    return { success: false };
  }
  return response as SaveFuncResponse;
};

export const DevService = {
  useCurrentGitSha,
  createBuiltinFunc,
  saveBuiltinFunc,
};
