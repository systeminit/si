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
import { ResourceHealth } from "@/api/sdf/dal/resource";

export interface ResourceSummaryForComponent {
  id: number;
  name: string;
  health: ResourceHealth;
  schema: string;
  resources: [];
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
    eventCheckedQualifications$.pipe(startWith(null)),
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
