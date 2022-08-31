import { combineLatest, shareReplay, map, switchMap } from "rxjs";
import { useObservable } from "@vueuse/rxjs";
import { visibility$ } from "@/observable/visibility";
import { ApiResponse, sdf } from "@/api/sdf";
import { GlobalErrorService } from "@/service/global_error";
import { system$ } from "@/observable/system";

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

export const DevService = {
  useCurrentGitSha,
};
