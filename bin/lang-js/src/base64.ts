import { Buffer } from "node:buffer";

export function base64Decode(encoded: string): string {
  return Buffer.from(encoded, "base64").toString("binary");
}
