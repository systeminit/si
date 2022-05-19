import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";
import { PropertyEditorValidations } from "@/api/sdf/dal/property_editor";

export interface GetPropertyEditorValidationsArgs {
  componentId: number;
  systemId: number;
}

export interface GetPropertyEditorValidationsRequest
  extends GetPropertyEditorValidationsArgs,
    Visibility {}

export type GetPropertyEditorValidationsResponse = PropertyEditorValidations;

const getPropertyEditorValidationsCollection: {
  [key: string]: Observable<ApiResponse<GetPropertyEditorValidationsResponse>>;
} = {};

export function getPropertyEditorValidations(
  args: GetPropertyEditorValidationsArgs,
): Observable<ApiResponse<GetPropertyEditorValidationsResponse>> {
  const cacheKey = `${args.componentId}-${args.systemId}`;
  if (getPropertyEditorValidationsCollection[cacheKey]) {
    return getPropertyEditorValidationsCollection[cacheKey];
  }
  getPropertyEditorValidationsCollection[cacheKey] = combineLatest([
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.get<ApiResponse<GetPropertyEditorValidationsResponse>>(
        "component/get_property_editor_validations",
        {
          ...args,
          ...visibility,
        },
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getPropertyEditorValidationsCollection[cacheKey];
}
