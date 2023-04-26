import ts from 'typescript';

// Note(paulo, zack): big old ass hacky hack, packaging lang-js creates some weird syntax issues with
// the @typescript/vfs library, whenever it tries to log to the console it crashes without a backtrace, with
// just a { kind: InvalidData, error: Error(\"expected value\", line: 1, column: 1) }
// which doesn't happen when running `npm run dev`
//
// Turns out they were using the same DEBUG env var we use for our logs, so disabling on import works
// We may want to rethink our env var to have it scoped, and we may need to fix the underlying issue
// but for now this gets us through, enabling ts as a language.
const oldDebug = process.env['DEBUG'];
delete process.env['DEBUG'];
import * as tsvfs from '@typescript/vfs'
process.env['DEBUG'] = oldDebug;

import Debug from "debug";

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
  }

  const fsMap = tsvfs.createDefaultMapFromNodeModules(compilerOptions);
  fsMap.set("index.ts", code);
  // TODO: have actual type here
  fsMap.set("types.d.ts", "type Input = any; type Output = any;");
  const system = tsvfs.createSystem(fsMap);

  const tsEnv = tsvfs.createVirtualTypeScriptEnvironment(system, Array.from(fsMap.keys()), ts, compilerOptions);
  return tsEnv.languageService.getEmitOutput("index.ts", false, true).outputFiles[0]?.text ?? "";
}
