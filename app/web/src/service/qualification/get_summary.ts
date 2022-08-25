import {
  combineLatest,
  map,
  Observable,
  shareReplay,
  startWith,
  switchMap,
} from "rxjs";
import { ApiResponse, sdf } from "@/api/sdf";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { GlobalErrorService } from "@/service/global_error";
import { eventCheckedQualifications$ } from "@/observable/qualification";

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
  getSummaryObservableCache = combineLatest([
    standardVisibilityTriggers$,
    eventCheckedQualifications$.pipe(startWith(null)),
  ]).pipe(
    switchMap(([[visibility]]) => {
      return sdf
        .get<ApiResponse<GetSummaryResponse>>("qualification/get_summary", {
          ...visibility,
        })
        .pipe(
          map((response: ApiResponse<GetSummaryResponse>) => {
            if (response.error) {
              GlobalErrorService.set(response);
              return undefined;
            }
            return response;
          }),
        );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getSummaryObservableCache;
}
