import { combineLatest, map, Observable, shareReplay, switchMap } from "rxjs";
import { ApiResponse, SDF } from "@/api/sdf";
import Bottle from "bottlejs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { GlobalErrorService } from "@/service/global_error";

export interface QualificationSummaryForComponent {
  componentId: number;
  componentName: string;
  total: number;
  succeeded: number;
  failed: number;
}

export interface GetSummaryResponse {
  total: number;
  succeeded: number;
  failed: number;
  components: QualificationSummaryForComponent[];
}

let getSummaryObservableCache: Observable<GetSummaryResponse | undefined>;

export function getSummary(): Observable<GetSummaryResponse | undefined> {
  if (getSummaryObservableCache) return getSummaryObservableCache;
  getSummaryObservableCache = combineLatest([standardVisibilityTriggers$]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf
        .get<ApiResponse<GetSummaryResponse>>("qualification/get_summary", {
          ...visibility,
        })
        .pipe(
          map((response: ApiResponse<GetSummaryResponse>) => {
            if (response.error) {
              GlobalErrorService.set(response);
              // If we encounter an error, return undefined.
              return undefined;
            }
            // Return the qualification summary information
            return response;
          }),
        );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getSummaryObservableCache;
}
