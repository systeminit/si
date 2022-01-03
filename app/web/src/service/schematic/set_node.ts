import { Visibility } from "@/api/sdf/dal/visibility";
import { from, Observable } from "rxjs";
import { ApiResponse } from "@/api/sdf";

export interface SetNodeArgs {
  name: String;
}

export interface SetNodeRequest extends SetNodeArgs, Visibility {}

export interface SetNodeResponse {
  schematic: string;
}

export function setNode(
  args: SetNodeArgs,
): Observable<ApiResponse<SetNodeResponse>> {
  return from([{ schematic: "node position updated" }]);
}
