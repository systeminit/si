import Bottle from "bottlejs";
import { Observable, tap } from "rxjs";
import { ApiResponse, SDF } from "@/api/sdf";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { changeSet$ } from "@/observable/change_set";

export interface UpdatedSelectedChangeSetArgs {
  nextChangeSetPk: number;
}

export type UpdatedSelectedChangeSetRequest = UpdatedSelectedChangeSetArgs;

interface UpdateSelectedChangeSetResponse {
  changeSet: ChangeSet;
}

export function updateSelectedChangeSet(
  args: UpdatedSelectedChangeSetArgs,
): Observable<ApiResponse<UpdateSelectedChangeSetResponse>> {
  const request: UpdatedSelectedChangeSetRequest = args;

  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return sdf
    .post<ApiResponse<UpdateSelectedChangeSetResponse>>(
      "change_set/update_selected_change_set",
      request,
    )
    .pipe(
      tap((response) => {
        if (!response.error) {
          changeSet$.next(response.changeSet);
        }
      }),
    );
}
