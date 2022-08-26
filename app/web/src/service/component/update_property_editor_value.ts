import Bottle from "bottlejs";
import { combineLatest, Observable, take } from "rxjs";
import { switchMap } from "rxjs/operators";
import _ from "lodash";
import { ApiResponse, SDF } from "@/api/sdf";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";
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
      return sdf.post<ApiResponse<UpdateFromEditFieldResponse>>(
        "component/update_property_editor_value",
        request,
      );
    }),
  );
}
