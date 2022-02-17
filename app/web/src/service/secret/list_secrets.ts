import { Secret } from "@/api/sdf/dal/secret";
import { ApiResponse, SDF } from "@/api/sdf";
import { combineLatest, combineLatestWith, from, Observable } from "rxjs";
import { standardVisibilityTriggers$ } from "@/observable/visibility";
import Bottle from "bottlejs";
import { switchMap } from "rxjs/operators";
import { workspace$ } from "@/observable/workspace";
import _ from "lodash";

export type ListSecretsResponse = Secret[];

export function listSecrets(): Observable<ApiResponse<ListSecretsResponse>> {
  return combineLatest([standardVisibilityTriggers$]).pipe(
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
      return sdf.get<ApiResponse<ListSecretsResponse>>("secret/list_secrets", {
        ...visibility,
      });
    }),
  );
}
