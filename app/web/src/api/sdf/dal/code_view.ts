export interface CodeView {
  language: CodeLanguage;
  code?: string;
}

// FIXME(nick): use this type in the CodeView interface once we want to dynamically check the code language type.
export type CodeLanguage = "diff" | "json" | "unknown" | "yaml" | "javascript";
