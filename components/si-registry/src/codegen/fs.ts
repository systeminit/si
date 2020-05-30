import {
  onExit,
  chunksToLinesAsync,
  streamWrite,
  streamEnd,
} from "@rauschma/stringio";
import childProcess from "child_process";
import fs from "fs";
import path from "path";
import XXHash from "xxhash";

export async function makePath(pathPart: string): Promise<string> {
  const absolutePathName = path.resolve(pathPart);
  if (!fs.existsSync(absolutePathName)) {
    await fs.promises.mkdir(absolutePathName, { recursive: true });
  }
  return absolutePathName;
}

export async function writeCode(filename: string, code: string): Promise<void> {
  const pathname = path.dirname(filename);
  const basename = path.basename(filename);
  const createdPath = await makePath(pathname);
  const codeFilename = path.join(createdPath, basename);
  let codeOutput = code;
  if (fs.existsSync(codeFilename)) {
    if (codeFilename.endsWith(".rs")) {
      // @ts-ignore - we know what this is, right? ;0
      const rustfmtChild = childProcess.spawn("rustfmt", ["--emit", "stdout"], {
        stdio: ["pipe", "pipe", "pipe"],
      });
      const exitPromise = onExit(rustfmtChild);
      await streamWrite(rustfmtChild.stdin, code);
      await streamEnd(rustfmtChild.stdin);
      codeOutput = "";
      for await (const line of chunksToLinesAsync(rustfmtChild.stdout)) {
        codeOutput += line;
      }
      await exitPromise;
    }
    const codeHash = XXHash.hash64(Buffer.from(codeOutput), 1234, "base64");
    const existingCode = await fs.promises.readFile(codeFilename);
    const existingCodeHash = XXHash.hash64(existingCode, 1234, "base64");
    if (codeHash == existingCodeHash) {
      return;
    }
  }
  await fs.promises.writeFile(codeFilename, codeOutput);
}
