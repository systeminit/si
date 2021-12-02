import { Visibility } from "@/api/sdf/dal/visibility";
import { from, Observable } from "rxjs";
import { ApiResponse } from "@/api/sdf";

export interface CreateConnectionArgs {
  name: String;
}

export interface CreateConnectionRequest
  extends CreateConnectionArgs,
    Visibility {}

export interface CreateConnectionResponse {
  schematic: string;
}

export function createConnection(
  args: CreateConnectionArgs,
): Observable<ApiResponse<CreateConnectionResponse>> {
  return from([{ schematic: "connection created" }]);
}
