import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, shareReplay, tap } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { EditFieldObjectKind, EditFields } from "@/api/sdf/dal/edit_field";

export interface GetEditFieldsArgs {
  objectKind: EditFieldObjectKind;
  id: number;
}

export interface GetEditFieldsRequest extends GetEditFieldsArgs, Visibility {}

export interface GetEditFieldsResponse {
  fields: EditFields;
}

const getEditFieldsCollection: {
  [objectKind: string]: {
    [key: string]: Observable<ApiResponse<GetEditFieldsResponse>>;
  };
} = {};

export function getEditFields(
  args: GetEditFieldsArgs,
): Observable<ApiResponse<GetEditFieldsResponse>> {
  if (
    getEditFieldsCollection[args.objectKind] &&
    getEditFieldsCollection[args.objectKind][args.id]
  ) {
    return getEditFieldsCollection[args.objectKind][args.id];
  }
  if (!getEditFieldsCollection[args.objectKind]) {
    getEditFieldsCollection[args.objectKind] = {};
  }
  getEditFieldsCollection[args.objectKind][args.id] = combineLatest([
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      const request: GetEditFieldsRequest = {
        ...args,
        ...visibility,
      };
      const response = sdf.get<ApiResponse<GetEditFieldsResponse>>(
        "edit_field/get_edit_fields",
        request,
      );
      return response;
    }),
    shareReplay(1),
  );
  return getEditFieldsCollection[args.objectKind][args.id];
}
