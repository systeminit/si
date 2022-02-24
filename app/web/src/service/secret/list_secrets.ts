import Bottle from "bottlejs";
import _ from "lodash";
import {
  combineLatest,
  combineLatestWith,
  from,
  Observable,
  share,
} from "rxjs";
import { switchMap } from "rxjs/operators";
import { ApiResponse, SDF } from "@/api/sdf";
import { Secret } from "@/api/sdf/dal/secret";
import { Visibility } from "@/api/sdf/dal/visibility";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import { workspace$ } from "@/observable/workspace";

export interface ListSecretRequest extends Visibility {
  workspaceId: number;
}

export interface ListSecretResponse {
  list: Secret[];
}

const secretList$ = combineLatest([standardVisibilityTriggers$]).pipe(
  combineLatestWith(workspace$),
  switchMap(([[[visibility]], workspace]) => {
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
    const request: ListSecretRequest = {
      ...visibility,
      workspaceId: workspace.id,
    };
    return sdf.get<ApiResponse<ListSecretResponse>>(
      "secret/list_secrets",
      request,
    );
  }),
  share(),
);

export function listSecrets(): Observable<ApiResponse<ListSecretResponse>> {
  return secretList$;
}
