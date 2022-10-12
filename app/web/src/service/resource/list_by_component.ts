import { combineLatest, map, Observable, shareReplay, switchMap } from "rxjs";
import { ApiResponse, sdf } from "@/api/sdf";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { GlobalErrorService } from "@/service/global_error";
import { ResourceHealth } from "@/api/sdf/dal/resource";
import { MockResource } from "../resource";

export interface ResourceSummaryForComponent {
  id: number;
  name: string;
  health: ResourceHealth;
  schema: string;
  resource?: MockResource;
}

export interface ListByComponentResponse {
  components: ResourceSummaryForComponent[];
}

let listByComponentObservableCache: Observable<
  ListByComponentResponse | undefined
>;

export function listByComponent(): Observable<
  ListByComponentResponse | undefined
> {
  if (listByComponentObservableCache) return listByComponentObservableCache;
  listByComponentObservableCache = combineLatest([
    standardVisibilityTriggers$,
    // removed this observable - does not seem to be the right event anyway
    // eventCheckedQualifications$.pipe(startWith(null)),
  ]).pipe(
    switchMap(([[visibility]]) => {
      return sdf
        .get<ApiResponse<ListByComponentResponse>>(
          "resource/list_resources_by_component",
          {
            ...visibility,
          },
        )
        .pipe(
          map((response: ApiResponse<ListByComponentResponse>) => {
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
  return listByComponentObservableCache;
}
