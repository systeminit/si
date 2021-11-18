import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { EditSession } from "@/api/sdf/dal/edit_session";
import { editSession$ } from "@/observable/edit_session";
import { editMode$ } from "@/observable/edit_mode";
import { Observable, tap } from "rxjs";

interface StartEditSessionRequest {
  changeSetPk: number;
}

interface StartEditSessionResponse {
  editSession: EditSession;
}

export function startEditSession(
  request: StartEditSessionRequest,
): Observable<ApiResponse<StartEditSessionResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return sdf
    .post<ApiResponse<StartEditSessionResponse>>(
      "change_set/start_edit_session",
      request,
    )
    .pipe(
      tap((response) => {
        if (!response.error) {
          editSession$.next(response.editSession);
          editMode$.next(true);
        }
      }),
    );
}
