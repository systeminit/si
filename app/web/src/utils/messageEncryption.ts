import { Base64 } from "js-base64";
import _sodium from "libsodium-wrappers";

import { PublicKey } from "@/store/secrets.store";

export async function encryptMessage(
  message: Record<string, string>,
  publicKey: PublicKey,
): Promise<number[]> {
  await _sodium.ready;
  const sodium = _sodium;

  // Base64 decode the key into a Uint8Array (i.e. "bytes")
  const pkey = Base64.toUint8Array(publicKey.public_key);

  return Array.from(sodium.crypto_box_seal(serializeMessage(message), pkey));
}

function serializeMessage(message: Record<string, string>): Uint8Array {
  const json = JSON.stringify(message, null, 0);
  const result = new Uint8Array(json.length);
  for (let i = 0; i < json.length; i++) {
    result[i] = json.charCodeAt(i);
  }
  return result;
}
