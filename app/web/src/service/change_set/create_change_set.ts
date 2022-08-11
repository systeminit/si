import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { changeSet$ } from "@/observable/change_set";
import { Observable, tap } from "rxjs";

interface CreateChangeSetArgs {
  changeSetName: string;
}

export type CreateChangeSetRequest = CreateChangeSetArgs;

interface CreateChangeSetResponse {
  changeSet: ChangeSet;
}

export function createChangeSet(
  args: CreateChangeSetArgs,
): Observable<ApiResponse<CreateChangeSetResponse>> {
  const request: CreateChangeSetRequest = args;
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return sdf
    .post<ApiResponse<CreateChangeSetResponse>>(
      "change_set/create_change_set",
      request,
    )
    .pipe(
      tap((response) => {
        if (!response.error) {
          changeSet$.next(response.changeSet);
        }
      }),
    );
}
