import Bottle from "bottlejs";
import { ApiResponse, SDF } from "@/api/sdf";
import { Workspace } from "@/api/sdf/dal/workspace";
import { Organization } from "@/api/sdf/dal/organization";
import { System } from "@/api/sdf/dal/system";
import { workspace$ } from "@/observable/workspace";
import { organization$ } from "@/observable/organization";
import { system$ } from "@/observable/system";
import { Observable, tap } from "rxjs";

interface GetDefaultsResponse {
  workspace: Workspace;
  organization: Organization;
  system: System;
}

export function getDefaults(): Observable<ApiResponse<GetDefaultsResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return sdf.get<ApiResponse<GetDefaultsResponse>>("session/get_defaults").pipe(
    tap((response) => {
      if (!response.error) {
        workspace$.next(response.workspace);
        organization$.next(response.organization);
        system$.next(response.system);
      }
    }),
  );
}
