import { ApiResponse, SDF } from "@/api/sdf";
import { lastValueFrom } from "rxjs";
import Bottle from "bottlejs";
import { PublicKey } from "@/api/sdf/dal/key_pair";

// The response _is_ the "PublicKey" from the DAL. You may need to decode the actual public key via "Base64.toUint8Array"
// or equivalent.
export type GetPublicKeyResponse = PublicKey;

/**
 * Returns the current `PublicKey` for the corresponding logged in billing
 * account
 *
 * **Note**: This is a blocking call, unlike most service calls--it does not
 * return an observable.
 */
export async function getPublicKeyRaw(): Promise<
  ApiResponse<GetPublicKeyResponse>
> {
  const bottle = Bottle.pop("default");
  const sdf: SDF = bottle.container.SDF;
  const reply$ = sdf.get<ApiResponse<GetPublicKeyResponse>>(
    "secret/get_public_key",
  );
  return await lastValueFrom(reply$);
}
