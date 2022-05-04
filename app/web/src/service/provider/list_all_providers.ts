import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, Observable, from } from "rxjs";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { Visibility } from "@/api/sdf/dal/visibility";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { InternalProvider } from "@/api/sdf/dal/provider";
import { ExternalProvider } from "@/api/sdf/dal/provider";
import { system$ } from "@/observable/system";

export interface ListAllProviderArgs extends Visibility {
  schemaVariantId: number;
}

export interface ListAllProviderRequest extends ListAllProviderArgs {
  workspaceId?: number;
}

export interface ListAllProviderResponse {
  internalProviders: InternalProvider[];
  externalProviders: ExternalProvider[];
}

export function listAllProviders(
  args: ListAllProviderRequest,
): Observable<ApiResponse<ListAllProviderResponse>> {
  return combineLatest([standardVisibilityTriggers$, system$, workspace$]).pipe(
    switchMap(([[visibility], system, workspace]) => {
      const bottle = Bottle.pop("default");
      const sdf: SDF = bottle.container.SDF;
      if (_.isNull(workspace)) {
        return from([
          {
            error: {
              statusCode: 10,
              message: "cannot make call without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      return sdf.get<ApiResponse<ListAllProviderResponse>>(
        "provider/list_all_providers",
        {
          ...args,
          ...visibility,
          systemId: system?.id,
          workspaceId: workspace.id,
        },
      );
    }),
  );
}
