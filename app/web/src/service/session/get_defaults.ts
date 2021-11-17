import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { Workspace } from "@/api/sdf/dal/workspace";
import { Organization } from "@/api/sdf/dal/organization";
import { workspace$ } from "@/observable/workspace";
import { organization$ } from "@/observable/organization";

interface GetDefaultsResponse {
  workspace: Workspace;
  organization: Organization;
}

export async function getDefaults(): Promise<ApiResponse<GetDefaultsResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const result: ApiResponse<GetDefaultsResponse> = await sdf.get(
    "session/get_defaults",
  );
  if (result.error) {
    return result;
  }
  workspace$.next(result.workspace);
  organization$.next(result.organization);
  return result;
}
