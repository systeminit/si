import { Base64 } from "js-base64";
import _sodium from "libsodium-wrappers";

export interface PublicKey {
  /**
   * The PK of the public key
   */
  pk: string;
  /**
   * The name of the public key
   */
  name: string;
  /**
   * The public key contents, encoded as a Base64 string
   *
   * # Examples
   *
   * Decoding a public key into a `Uint8Array` type:
   *
   * ```ts
   * Base64.toUint8Array(key.public_key);
   * ```
   */
  public_key: string;
  /**
   * A created lamport clock, used to sort multiple generations of key pairs
   */
  created_lamport_clock: string;

  created_at: string;
  updated_at: string;
}

export async function encryptMessage(message: Record<string, string>, publicKey: PublicKey): Promise<number[]> {
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
