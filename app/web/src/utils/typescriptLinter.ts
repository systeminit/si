import {
  createSystem,
  createDefaultMapFromCDN,
  createVirtualTypeScriptEnvironment,
  VirtualTypeScriptEnvironment,
} from "@typescript/vfs";
import { Extension } from "@codemirror/state";
import { completeFromList, autocompletion } from "@codemirror/autocomplete";
import { EditorView, Tooltip } from "@codemirror/view";
import { Diagnostic } from "@codemirror/lint";
import { DiagnosticMessageChain, System, CompilerOptions, displayPartsToString } from "typescript";
import { snippets } from "./typescriptLinterSnippets";
import type { CompletionContext } from "@codemirror/autocomplete";

export type AsyncLintSource = (view: EditorView) => Promise<readonly Diagnostic[]>;

export type HoverTooltipSource = (view: EditorView, pos: number) => Tooltip | null;

export type RemoveTooltipOnUpdateSource = (codeTooltip: CodeTooltip) => Extension;

export interface TypescriptSource {
  lintSource: AsyncLintSource;
  autocomplete: Extension;
  hoverTooltipSource: HoverTooltipSource;
  removeTooltipOnUpdateSource: RemoveTooltipOnUpdateSource;
}

export interface CodeTooltip {
  currentTooltip: Tooltip | null;
  destroy: () => void;
  update: () => void;
}

const tsVersion = "4.7.4";
const useCache = true;
let fsMap: Map<string, string>;
let vfsSystem: System;

const defaultFilename = "index.ts";
// If the document gets blanked out, typescript still needs something.
const fallbackCode = "console.log('foo')";
let tsEnv: VirtualTypeScriptEnvironment;

export const createTypescriptSource = async (types: string): Promise<TypescriptSource> => {
  // we lazy load typescript to help speed things up
  const ts = await import("typescript");

  // TODO: do we provide all DOM types? its used for console and possibly fetch, but it may define types that don't exist in lang-js
  const compilerOptions: CompilerOptions = {
    target: ts.ScriptTarget.ES2020,
    noUncheckedIndexedAccess: true,
    lib: ["ES2020", "DOM"],
    allowJs: true,
    checkJs: true,
    strict: false,
  };

  if (!fsMap && !vfsSystem) {
    fsMap = await createDefaultMapFromCDN(compilerOptions, tsVersion, useCache, ts);
    vfsSystem = createSystem(fsMap);
  }

  fsMap.set(defaultFilename, fallbackCode);
  fsMap.set("func.d.ts", types);
  fsMap.set("lodash.d.ts", "declare var _: any");

  tsEnv = createVirtualTypeScriptEnvironment(
    vfsSystem,
    [defaultFilename, "func.d.ts", "lodash.d.ts"],
    ts,
    compilerOptions,
  );

  const autocomplete = autocompletion({
    icons: true,
    override: [
      (ctx: CompletionContext) => {
        const completions = tsEnv.languageService.getCompletionsAtPosition(defaultFilename, ctx.pos, {});

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

  const hoverTooltipSource = (view: EditorView, pos: number): Tooltip | null => {
    return GetTooltipFromPos(pos);
  };

  const lintSource = async (view: EditorView) => {
    const doc = view.state.doc;
    // We could be more efficient by updating only the changed spans
    const docString = doc.toString();

    let diagnostics: Diagnostic[] = [];

    // custom lint rule to ensure that we have a `main` function entrypoint
    if (!docString.includes("function main(")) {
      diagnostics = diagnostics.concat(
        diagnosticsForMessage(1, 1, "Function should include a `main` function for code execution"),
      );
    }

    tsEnv.updateFile(defaultFilename, docString.trim().length === 0 ? fallbackCode : docString);

    for (const tsDiagnostic of tsEnv.languageService.getSyntacticDiagnostics(defaultFilename)) {
      const from = tsDiagnostic.start;
      const to = from + tsDiagnostic.length;
      diagnostics = diagnostics.concat(diagnosticsForMessage(from, to, tsDiagnostic.messageText));
    }
    for (const tsDiagnostic of tsEnv.languageService.getSemanticDiagnostics(defaultFilename)) {
      // Note(paulo): I'm not sure if this is the correct way of doing this
      const from = tsDiagnostic.start ?? 0;
      const to = from + (tsDiagnostic.length ?? 0);
      diagnostics = diagnostics.concat(diagnosticsForMessage(from, to, tsDiagnostic.messageText));
    }
    for (const tsDiagnostic of tsEnv.languageService.getSuggestionDiagnostics(defaultFilename)) {
      const from = tsDiagnostic.start;
      const to = from + tsDiagnostic.length;
      diagnostics = diagnostics.concat(diagnosticsForMessage(from, to, tsDiagnostic.messageText));
    }

    return diagnostics;
  };

  const removeTooltipOnUpdateSource = (codeTooltip: CodeTooltip): Extension => {
    return EditorView.updateListener.of((update) => {
      if (codeTooltip.currentTooltip && update.startState.selection.main.head !== update.state.selection.main.head) {
        codeTooltip.destroy();
      }
    });
  };

  return {
    lintSource,
    autocomplete,
    hoverTooltipSource,
    removeTooltipOnUpdateSource,
  };
};

function diagnosticsForMessage(from: number, to: number, message: string | DiagnosticMessageChain): Diagnostic[] {
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
    messages = messages.concat(diagnosticsForMessage(from, to, message.messageText));
    for (const nextInChain of message.next ?? []) {
      messages = messages.concat(diagnosticsForMessage(from, to, nextInChain.messageText));
    }
    return messages;
  }
}

export function GetTooltipFromPos(pos: number): Tooltip | null {
  const quickInfo = tsEnv.languageService.getQuickInfoAtPosition(defaultFilename, pos);
  if (!quickInfo) {
    return null;
  }

  const parts = `<div class="cm-tooltip-doc-signature">${displayPartsToString(quickInfo.displayParts)}</div>`;

  const docs = quickInfo.documentation?.length
    ? `<div class="cm-tooltip-doc-details">${displayPartsToString(quickInfo?.documentation)}</div>`
    : "";

  const tags = quickInfo.tags?.length
    ? quickInfo?.tags
        ?.map(function format(t) {
          let tag = `<div class="cm-tooltip-doc-tag"><span class="cm-tooltip-doc-tag-name">@${
            t.name
          }:</span> <span class="cm-tooltip-doc-tag-info">${displayPartsToString(t.text)}</span></div>`;
          if (t.name === "example") {
            tag = `<div class="cm-tooltip-doc-tag"><span class="cm-tooltip-doc-tag-name">@${
              t.name
            }:</span>\n <span class="cm-tooltip-doc-tag-example">${displayPartsToString(t.text)}</span></div>`;
          }
          return tag;
        })
        .join("")
    : "";

  return {
    pos: quickInfo.textSpan.start,
    end: quickInfo.textSpan.start + quickInfo.textSpan.length,
    create() {
      const dom = document.createElement("div");
      dom.innerHTML = parts + docs + tags;
      return {
        dom,
        destroy: () => {
          dom.remove();
        },
      };
    },
    above: false,
  };
}

export function nonNullable<T>(value: T): value is NonNullable<T> {
  return value !== null && value !== undefined;
}
