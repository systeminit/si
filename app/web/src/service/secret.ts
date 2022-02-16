import { listSecretKinds } from "./secret/list_secret_kinds";
import { createSecret } from "./secret/create_secret";
import { listSecrets } from "./secret/list_secrets";

export const SecretService = {
  createSecret,
  listSecrets,
  listSecretKinds,
};
