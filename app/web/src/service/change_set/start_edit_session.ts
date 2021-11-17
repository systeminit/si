import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { EditSession } from "@/api/sdf/dal/edit_session";
import { editSession$ } from "@/observable/edit_session";
import { editMode$ } from "@/observable/edit_mode";

interface StartEditSessionRequest {
  changeSetPk: number;
}

interface StartEditSessionResponse {
  editSession: EditSession;
}

export async function startEditSession(
  request: StartEditSessionRequest,
): Promise<ApiResponse<StartEditSessionResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const response: ApiResponse<StartEditSessionResponse> = await sdf.post(
    "change_set/start_edit_session",
    request,
  );
  if (response.error) {
    return response;
  }
  editSession$.next(response.editSession);
  editMode$.next(true);
  return response;
}
