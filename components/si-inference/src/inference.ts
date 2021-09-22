export interface SelectEntityId {
  entityId: string;
  entityType?: never;
  targetEntity?: never;
  self?: never;
  data: From;
}

export interface SelectEntityType {
  entityId?: never;
  entityType: string;
  targetEntity?: never;
  self?: never;
  data: From;
}

export interface SelectTarget {
  entityId?: never;
  entityType?: never;
  targetEntity: true;
  self?: never;
  data: From;
}

export interface SelectSelf {
  entityId?: never;
  entityType?: never;
  targetEntity?: never;
  self: true;
  data: From;
}

export type SelectOptions =
  | SelectEntityType
  | SelectEntityId
  | SelectTarget
  | SelectSelf;

export type Select = SelectOptions[];

export interface FromEntryName {
  name: true;
  path?: never;
}

export interface FromEntryPath {
  name?: never;
  path: string[];
}

export type FromEntry = FromEntryName | FromEntryPath;

export type From = FromEntry | Array<FromEntry>;

export interface ToEntryName {
  name: true;
  path?: never;
  extraPath?: never;
}

export interface ToEntryPath {
  name?: never;
  path: string[];
  extraPath?: string[];
}

export type ToEntry = ToEntryName | ToEntryPath;

export type To = ToEntry;

export interface InferenceBase {
  kind: string;
  name: string;
  from: Select;
  to: To;
  if?: string;
}

export interface InferenceLambda extends InferenceBase {
  kind: "lambda";
  code: string;
}

export type Inference = InferenceLambda;
