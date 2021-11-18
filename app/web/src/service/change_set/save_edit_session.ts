import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { EditSession } from "@/api/sdf/dal/edit_session";
import { editSession$ } from "@/observable/edit_session";
import { editMode$ } from "@/observable/edit_mode";
import { from, mergeMap, Observable, take, tap } from "rxjs";

/**
 * Returns the edit session that has been saved, or null if no
 * edit session existed to save in the first place.
 */
interface SaveEditSessionResponse {
  editSession: EditSession | null;
}

export function saveEditSession(): Observable<
  ApiResponse<SaveEditSessionResponse>
> {
  return editSession$.pipe(
    take(1),
    mergeMap((editSession) => {
      if (editSession) {
        const bottle = Bottle.pop("default");
        const sdf: SDF = bottle.container.SDF;
        return from(
          sdf.post<SaveEditSessionResponse>("change_set/save_edit_session", {
            editSessionPk: editSession.pk,
          }),
        );
      } else {
        const response: ApiResponse<SaveEditSessionResponse> = {
          editSession: null,
        };
        return from([response]);
      }
    }),
    tap((response) => {
      if (!response.error) {
        editSession$.next(null);
        editMode$.next(false);
      }
    }),
    take(1),
  );
}
