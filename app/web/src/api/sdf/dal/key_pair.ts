import { StandardModel } from "@/api/sdf/dal/standard_model";

export interface PublicKey extends StandardModel {
  name: string;
  // This is base64 encoded. The consumer may need to execute "Base64.toUint8Array(<public-key>)" to get the key.
  public_key: string;
  created_lampot_clock: string;
}
