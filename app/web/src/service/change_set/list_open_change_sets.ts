import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { LabelList } from "@/api/sdf/dal/label_list";

interface ListOpenChangesetsResponse {
  list: LabelList<number>;
}

export async function listOpenChangeSets(): Promise<
  ApiResponse<ListOpenChangesetsResponse>
> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const result: ApiResponse<ListOpenChangesetsResponse> = await sdf.get(
    "change_set/list_open_change_sets",
  );
  if (result.error) {
    return result;
  }
  return result;
}
