import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { changeSet$, revision$ } from "@/observable/change_set";
import { editSession$ } from "@/observable/edit_session";
import { editMode$ } from "@/observable/edit_mode";

interface GetChangeSetRequest {
  pk: number;
}

interface GetChangeSetResponse {
  changeSet: ChangeSet;
}

export async function getChangeSet(
  request: GetChangeSetRequest,
): Promise<ApiResponse<GetChangeSetResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const response: ApiResponse<GetChangeSetResponse> = await sdf.get(
    "change_set/get_change_set",
    request,
  );
  if (response.error) {
    return response;
  }
  changeSet$.next(response.changeSet);
  editSession$.next(null);
  revision$.next(null);
  editMode$.next(false);

  return response;
}
