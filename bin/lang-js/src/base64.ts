import ts from "typescript";
import * as tsvfs from "@typescript/vfs";
import { Debug } from "./debug";

const debug = Debug("langJs:base64");

export function base64Decode(encoded: string): string {
  return Buffer.from(encoded, "base64").toString("binary");
}

export function base64ToJs(encoded: string): string {
  const code = base64Decode(encoded);
  debug({ code });

  const compilerOptions = {
    target: ts.ScriptTarget.ES2020,
    lib: ["ES2020", "DOM"],
  };

  const fsMap = tsvfs.createDefaultMapFromNodeModules(compilerOptions);
  fsMap.set("index.ts", code);
  // TODO: have actual type here
  fsMap.set("types.d.ts", "type Input = any; type Output = any;");
  const system = tsvfs.createSystem(fsMap);

  const tsEnv = tsvfs.createVirtualTypeScriptEnvironment(
    system,
    Array.from(fsMap.keys()),
    ts,
    compilerOptions,
  );
  return (
    tsEnv.languageService.getEmitOutput("index.ts", false, true).outputFiles[0]
      ?.text ?? ""
  );
}
