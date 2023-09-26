import {
  createSystem,
  createDefaultMapFromCDN,
  createVirtualTypeScriptEnvironment,
} from "@typescript/vfs";
import { Extension } from "@codemirror/state";
import { completeFromList, autocompletion } from "@codemirror/autocomplete";
import { EditorView } from "@codemirror/view";
import { Diagnostic } from "@codemirror/lint";
import { DiagnosticMessageChain, System, CompilerOptions } from "typescript";
import { snippets } from "./typescriptLinterSnippets";
import type { CompletionContext } from "@codemirror/autocomplete";

export type AsyncLintSource = (
  view: EditorView,
) => Promise<readonly Diagnostic[]>;

export interface TypescriptSource {
  lintSource: AsyncLintSource;
  autocomplete: Extension;
}

const tsVersion = "4.7.4";
const useCache = true;
let fsMap: Map<string, string>;
let vfsSystem: System;

const defaultFilename = "index.ts";
// If the document gets blanked out, typescript still needs something.
const fallbackCode = "console.log('foo')";

export const createTypescriptSource = async (
  types: string,
): Promise<TypescriptSource> => {
  // we lazy load typescript to help speed things up
  const ts = await import("typescript");

  // TODO: do we provide all DOM types? its used for console and possibly fetch, but it may define types that don't exist in lang-js
  const compilerOptions: CompilerOptions = {
    target: ts.ScriptTarget.ES2020,
    noUncheckedIndexedAccess: true,
    lib: ["ES2020", "DOM"],
  };

  if (!fsMap && !vfsSystem) {
    fsMap = await createDefaultMapFromCDN(
      compilerOptions,
      tsVersion,
      useCache,
      ts,
    );
    vfsSystem = createSystem(fsMap);
  }

  fsMap.set(defaultFilename, fallbackCode);
  fsMap.set("func.d.ts", types);

  const tsEnv = createVirtualTypeScriptEnvironment(
    vfsSystem,
    [defaultFilename, "func.d.ts"],
    ts,
    compilerOptions,
  );

  const autocomplete = autocompletion({
    icons: true,
    override: [
      (ctx: CompletionContext) => {
        const completions = tsEnv.languageService.getCompletionsAtPosition(
          defaultFilename,
          ctx.pos,
          {},
        );

        const completionsWithSnippets = [
          ...snippets,
          ...(completions?.entries.map((c) => ({
            type: "function",
            label: c.name,
          })) ?? []),
        ];

        return completeFromList(completionsWithSnippets)(ctx);
      },
    ],
  });

  const lintSource = async (view: EditorView) => {
    const doc = view.state.doc;
    // We could be more efficient by updating only the changed spans
    const docString = doc.toString();
    tsEnv.updateFile(
      defaultFilename,
      docString.trim().length === 0 ? fallbackCode : docString,
    );

    // TODO: use getQuickInfoAtPosition to display the types definitions on hover
    // https://github.com/microsoft/TypeScript/blob/0f724c04308e20d93d397e82b11f82ad6f810c44/src/services/types.ts#L433

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
    for (const tsDiagnostic of tsEnv.languageService.getSemanticDiagnostics(
      defaultFilename,
    )) {
      // Note(paulo): I'm not sure if this is the correct way of doing this
      const from = tsDiagnostic.start ?? 0;
      const to = from + (tsDiagnostic.length ?? 0);
      diagnostics = diagnostics.concat(
        diagnosticsForMessage(from, to, tsDiagnostic.messageText),
      );
    }
    for (const tsDiagnostic of tsEnv.languageService.getSuggestionDiagnostics(
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

  return { lintSource, autocomplete };
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
