// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { LeafInputLocation } from "./LeafInputLocation";
import type { LeafKind } from "./LeafKind";

export type LeafFunctionSpec = {
  funcUniqueId: string;
  leafKind: LeafKind;
  uniqueId: string | null;
  deleted: boolean;
  inputs: Array<LeafInputLocation>;
};