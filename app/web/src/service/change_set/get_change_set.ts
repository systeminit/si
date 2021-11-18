import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { changeSet$, revision$ } from "@/observable/change_set";
import { editSession$ } from "@/observable/edit_session";
import { editMode$ } from "@/observable/edit_mode";
import { Observable, tap } from "rxjs";

interface GetChangeSetRequest {
  pk: number;
}

interface GetChangeSetResponse {
  changeSet: ChangeSet;
}

export function getChangeSet(
  request: GetChangeSetRequest,
): Observable<ApiResponse<GetChangeSetResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return sdf
    .get<ApiResponse<GetChangeSetResponse>>(
      "change_set/get_change_set",
      request,
    )
    .pipe(
      tap((response) => {
        if (!response.error) {
          changeSet$.next(response.changeSet);
          editSession$.next(null);
          revision$.next(null);
          editMode$.next(false);
        }
      }),
    );
}
