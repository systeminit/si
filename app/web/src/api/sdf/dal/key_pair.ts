import { StandardModel } from "@/api/sdf/dal/standard_model";

/**
 * A public key with metadata, used to encrypt secrets
 */
export interface PublicKey extends StandardModel {
  /**
   * The ID of the public key
   */
  id: number;
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
  created_lampot_clock: string;
}
