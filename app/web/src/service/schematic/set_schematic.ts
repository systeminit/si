import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { Schematic } from "@/api/sdf/dal/schematic";
import { Observable, take, tap } from "rxjs";
import { Visibility } from "@/api/sdf/dal/visibility";
import { visibility$ } from "@/observable/visibility";
import { switchMap } from "rxjs/operators";
import { editSessionWritten$ } from "@/observable/edit_session";

export interface SetSchematicArgs {
  name: string;
}

export interface SetSchematicRequest extends SetSchematicArgs, Visibility {}

export interface SetSchematicResponse {
  schematic: Schematic;
}

export function setSchematic(
  args: SetSchematicArgs,
): Observable<ApiResponse<SetSchematicResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return visibility$.pipe(
    take(1),
    switchMap((visibility) => {
      return sdf
        .post<ApiResponse<SetSchematicResponse>>("schematic/set_schematic", {
          ...args,
          ...visibility,
        })
        .pipe(
          tap((response) => {
            if (!response.error) {
              editSessionWritten$.next(true);
            }
          }),
        );
    }),
  );
}
