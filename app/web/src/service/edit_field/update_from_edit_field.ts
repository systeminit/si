import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { Observable, take, tap } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { EditFieldObjectKind } from "@/api/sdf/dal/edit_field";
import { editSessionWritten$ } from "@/observable/edit_session";

export interface UpdateFromEditFieldArgs {
  objectKind: EditFieldObjectKind;
  objectId: number;
  editFieldId: string;
  value: any;
}

export interface UpdateFromEditFieldRequest
  extends UpdateFromEditFieldArgs,
    Visibility {}

export interface UpdateFromEditFieldResponse {
  success: boolean;
}

export function updateFromEditField(
  args: UpdateFromEditFieldArgs,
): Observable<ApiResponse<UpdateFromEditFieldResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return visibility$.pipe(
    take(1),
    switchMap((visibility) => {
      const request: UpdateFromEditFieldRequest = {
        ...args,
        ...visibility,
      };
      return sdf
        .post<ApiResponse<UpdateFromEditFieldResponse>>(
          "edit_field/update_from_edit_field",
          request,
        )
        .pipe(
          tap((response) => {
            editSessionWritten$.next(true);
            console.log({ response });
          }),
        );
    }),
  );
}
