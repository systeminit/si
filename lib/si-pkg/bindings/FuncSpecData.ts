// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { FuncSpecBackendKind } from "./FuncSpecBackendKind";
import type { FuncSpecBackendResponseType } from "./FuncSpecBackendResponseType";

export type FuncSpecData = {
  name: string;
  displayName: string | null;
  description: string | null;
  handler: string;
  codeBase64: string;
  backendKind: FuncSpecBackendKind;
  responseType: FuncSpecBackendResponseType;
  hidden: boolean;
  link: string | null;
};