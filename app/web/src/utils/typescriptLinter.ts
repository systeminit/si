import { DiagnosticMessageChain, System } from "typescript";
import {
  createSystem,
  createDefaultMapFromCDN,
  createVirtualTypeScriptEnvironment,
} from "@typescript/vfs";
import { EditorView } from "@codemirror/view";
import { Diagnostic } from "@codemirror/lint";

export type AsyncLintSource = (
  view: EditorView,
) => Promise<readonly Diagnostic[]>;

const tsVersion = "4.7.4";
const useCache = true;
let fsMap: Map<string, string>;
let vfsSystem: System;

export const createLintSource = async (): Promise<AsyncLintSource> => {
  // we lazy load typescript to help speed things up
  const ts = await import("typescript");

  if (!fsMap && !vfsSystem) {
    fsMap = await createDefaultMapFromCDN(
      { target: ts.ScriptTarget.ES2015 },
      tsVersion,
      useCache,
      ts,
    );
    vfsSystem = createSystem(fsMap);
  }

  const defaultFilename = "index.ts";
  // We need something, anything valid in here get the environment going
  fsMap.set(defaultFilename, "console.log('foo')");

  const tsEnv = createVirtualTypeScriptEnvironment(
    vfsSystem,
    [defaultFilename],
    ts,
  );

  return async (view: EditorView) => {
    const doc = view.state.doc;
    // We could be more efficient by updating only the changed spans
    tsEnv.updateFile(defaultFilename, doc.toString());

    let diagnostics: Diagnostic[] = [];
    for (const tsDiagnostic of tsEnv.languageService.getSyntacticDiagnostics(
      defaultFilename,
    )) {
      const from = tsDiagnostic.start;
      const to = from + tsDiagnostic.length;
      diagnostics = diagnostics.concat(
        diagnosticsForMessage(from, to, tsDiagnostic.messageText),
      );
    }

    return diagnostics;
  };
};

function diagnosticsForMessage(
  from: number,
  to: number,
  message: string | DiagnosticMessageChain,
): Diagnostic[] {
  if (typeof message === "string") {
    return [
      {
        from,
        to,
        severity: "error",
        source: "tsserver",
        message,
      },
    ];
  } else {
    let messages: Diagnostic[] = [];
    messages = messages.concat(
      diagnosticsForMessage(from, to, message.messageText),
    );
    for (const nextInChain of message.next ?? []) {
      messages = messages.concat(
        diagnosticsForMessage(from, to, nextInChain.messageText),
      );
    }
    return messages;
  }
}
