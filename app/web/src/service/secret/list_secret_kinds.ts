import { ApiResponse } from "@/api/sdf";
import { SecretKind, SecretKindFields } from "@/api/sdf/dal/secret";
import { Observable } from "rxjs";

export interface ListSecretKindFieldsResponse {
  fields: SecretKindFields[];
}

export function listSecretKindFields(): Observable<
  ApiResponse<ListSecretKindFieldsResponse>
> {
  return new Observable((subscriber) => {
    subscriber.next({
      fields: [
        {
          secretKind: SecretKind.DockerHub,
          displayName: "Docker Hub",
          fields: [
            {
              keyName: "username",
              displayName: "Docker Hub Username",
              password: false,
            },
            {
              keyName: "password",
              displayName: "Docker Hub Password",
              password: true,
            },
          ],
        },
      ],
    });
  });
}
