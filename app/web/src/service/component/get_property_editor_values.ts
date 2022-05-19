import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, shareReplay } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import _ from "lodash";
import { PropertyEditorValues } from "@/api/sdf/dal/property_editor";

export interface GetPropertyEditorValuesArgs {
  componentId: number;
  systemId: number;
}

export interface GetPropertyEditorValuesRequest
  extends GetPropertyEditorValuesArgs,
    Visibility {}

export type GetPropertyEditorValuesResponse = PropertyEditorValues;

const getPropertyEditorValuesCollection: {
  [key: string]: Observable<ApiResponse<GetPropertyEditorValuesResponse>>;
} = {};

export function getPropertyEditorValues(
  args: GetPropertyEditorValuesArgs,
): Observable<ApiResponse<GetPropertyEditorValuesResponse>> {
  const cacheKey = `${args.componentId}-${args.systemId}`;
  if (getPropertyEditorValuesCollection[cacheKey]) {
    return getPropertyEditorValuesCollection[cacheKey];
  }
  getPropertyEditorValuesCollection[cacheKey] = combineLatest([
    standardVisibilityTriggers$,
  ]).pipe(
    switchMap(([[visibility]]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      return sdf.get<ApiResponse<GetPropertyEditorValuesResponse>>(
        "component/get_property_editor_values",
        {
          ...args,
          ...visibility,
        },
      );
    }),
    shareReplay({ bufferSize: 1, refCount: true }),
  );
  return getPropertyEditorValuesCollection[cacheKey];
}
