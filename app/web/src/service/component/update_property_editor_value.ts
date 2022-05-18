import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, take, tap } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { editSessionWritten$ } from "@/observable/edit_session";
import _ from "lodash";
import { AttributeContext } from "@/api/sdf/dal/attribute";

export interface UpdatePropertyEditorValueArgs {
  attributeValueId: number;
  parentAttributeValueId?: number;
  attributeContext: AttributeContext;
  value?: unknown;
  key?: string;
}

export interface UpdatePropertyEditorValueRequest
  extends UpdatePropertyEditorValueArgs,
    Visibility {}

export interface UpdateFromEditFieldResponse {
  success: boolean;
}

export function updateFromEditField(
  args: UpdatePropertyEditorValueArgs,
): Observable<ApiResponse<UpdateFromEditFieldResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return combineLatest([visibility$]).pipe(
    take(1),
    switchMap(([visibility]) => {
      const request: UpdatePropertyEditorValueRequest = {
        ...args,
        ...visibility,
      };
      return sdf
        .post<ApiResponse<UpdateFromEditFieldResponse>>(
          "component/update_property_editor_value",
          request,
        )
        .pipe(
          tap((_response) => {
            editSessionWritten$.next(true);
          }),
        );
    }),
  );
}
