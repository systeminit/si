import Bottle from "bottlejs";
import { combineLatest, switchMap, Observable, shareReplay } from "rxjs";
import { ApiResponse, SDF } from "@/api/sdf";
import { ComponentStats } from "@/api/sdf/dal/change_set";
import { standardVisibilityTriggers$ } from "@/observable/visibility";

interface GetStatsResponse {
  componentStats: ComponentStats;
}

const changeSetStats$ = combineLatest([standardVisibilityTriggers$]).pipe(
  switchMap(([[visibility]]) => {
    const bottle = Bottle.pop("default");
    const sdf: SDF = bottle.container.SDF;
    return sdf.get<ApiResponse<GetStatsResponse>>("change_set/get_stats", {
      ...visibility,
    });
  }),
  shareReplay({ bufferSize: 1, refCount: true }),
);

export function getStats(): Observable<ApiResponse<GetStatsResponse>> {
  return changeSetStats$;
}
