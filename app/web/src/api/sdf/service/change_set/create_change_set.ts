import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { ChangeSet } from "@/api/sdf/dal/change_set";
import { EditSession } from "@/api/sdf/dal/edit_session";
import { changeSet$, eventChangeSetCreated$ } from "@/observable/change_set";
import { editSession$ } from "@/observable/edit_session";
import { editMode$ } from "@/observable/edit_mode";

interface CreateChangeSetRequest {
  changeSetName: string;
}

interface CreateChangeSetResponse {
  changeSet: ChangeSet;
  editSession: EditSession;
}

export async function createChangeSet(
  request: CreateChangeSetRequest,
): Promise<ApiResponse<CreateChangeSetResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const response: ApiResponse<CreateChangeSetResponse> = await sdf.post(
    "change_set/create_change_set",
    request,
  );
  if (response.error) {
    return response;
  }
  changeSet$.next(response.changeSet);
  editSession$.next(response.editSession);
  editMode$.next(true);
  eventChangeSetCreated$.next(response.changeSet.pk);
  return response;
}
