import Bottle from "bottlejs";
import _ from "lodash";
import { combineLatestWith, from, Observable, take, tap } from "rxjs";
import { switchMap } from "rxjs/operators";
import { ApiResponse, SDF } from "@/api/sdf";
import {
  Secret,
  SecretAlgorithm,
  SecretKind,
  SecretObjectType,
  SecretVersion,
} from "@/api/sdf/dal/secret";
import { Visibility } from "@/api/sdf/dal/visibility";
import { editSessionWritten$ } from "@/observable/edit_session";
import { visibility$ } from "@/observable/visibility";
import { workspace$ } from "@/observable/workspace";

export interface CreateSecretArgs {
  name: string;
  objectType: SecretObjectType;
  kind: SecretKind;
  crypted: number[];
  keyPairId: number;
  version: SecretVersion;
  algorithm: SecretAlgorithm;
}

export interface CreateSecretRequest extends CreateSecretArgs, Visibility {
  workspaceId: number;
}

export interface CreateSecretResponse {
  secret: Secret;
}
export function createSecret(
  args: CreateSecretArgs,
): Observable<ApiResponse<CreateSecretResponse>> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  return visibility$.pipe(
    take(1),
    combineLatestWith(workspace$),
    switchMap(([visibility, workspace]) => {
      if (_.isNull(workspace)) {
        return from([
          {
            error: {
              statusCode: 10,
              message: "cannot create an application without a workspace; bug!",
              code: 10,
            },
          },
        ]);
      }
      const request: CreateSecretRequest = {
        ...args,
        ...visibility,
        workspaceId: workspace.id,
      };
      return sdf
        .post<ApiResponse<CreateSecretResponse>>(
          "secret/create_secret",
          request,
        )
        .pipe(
          tap((response) => {
            if (!response.error) {
              editSessionWritten$.next(true);
            }
          }),
        );
    }),
  );
}
