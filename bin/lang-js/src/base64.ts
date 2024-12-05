import { Buffer } from "node:buffer";
import { Debug } from "./debug.ts";
import { transpile } from "jsr:@deno/emit";

const debug = Debug("langJs:base64");

export function base64Decode(encoded: string): string {
  return Buffer.from(encoded, "base64").toString("binary");
}

export async function base64ToJs(encoded: string): Promise<string> {
  const code = base64Decode(encoded);

  debug({ code });

  const tempDir = await Deno.makeTempDir();
  const tempFile = `${tempDir}/script.ts`;
  await Deno.writeTextFile(tempFile, code);
  const fileUrl = new URL(tempFile, import.meta.url);
  return (await transpile(fileUrl)).get(fileUrl.href) as string;
}
