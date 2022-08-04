import { combineLatest, Observable, shareReplay, switchMap } from "rxjs";
import { ApiResponse, SDF } from "@/api/sdf";
import Bottle from "bottlejs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";

export interface QualificationSummaryForComponent {
  component_id: number;
  component_name: string;
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

export function getSummary(): Observable<ApiResponse<GetSummaryResponse>> {
  return combineLatest([standardVisibilityTriggers$]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.get<ApiResponse<GetSummaryResponse>>(
        "qualification/get_summary",
        {
          ...visibility,
        },
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
}
