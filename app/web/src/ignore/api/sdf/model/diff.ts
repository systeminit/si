import _ from "lodash";

export interface DiffEntryEdit {
  path: string[];
  before: any;
  after: any;
}

export interface DiffEntryAdd {
  path: string[];
  after: any;
}

export interface DiffEntryRemove {
  path: string[];
  before: any;
}

export interface DiffEntryRepeatedSize {
  path: string[];
  size: number;
}

export interface DiffEntryAddValue {
  edit?: never;
  add: DiffEntryAdd;
  remove?: never;
  repeatedSize?: never;
}

export interface DiffEntryEditValue {
  edit: DiffEntryEdit;
  add?: never;
  remove?: never;
  repeatedSize?: never;
}

export interface DiffEntryRemoveValue {
  edit?: never;
  add?: never;
  remove: DiffEntryRemove;
  repeatedSize?: never;
}

export interface DiffEntryRepeatedSize {
  edit?: never;
  add?: never;
  remove?: never;
  repeatedSize: DiffEntryRepeatedSize;
}

export type DiffEntry =
  | DiffEntryAddValue
  | DiffEntryEditValue
  | DiffEntryRemoveValue
  | DiffEntryRepeatedSize;

export type Diff = DiffEntry[];

export function hasDiff(diff: Diff, path: string[]): boolean {
  const hasDiff = _.find(diff, de => {
    if (de.add) {
      return _.isEqual(de.add.path, path);
    } else if (de.edit) {
      return _.isEqual(de.edit.path, path);
    } else {
      return false;
    }
  });
  if (hasDiff) {
    return true;
  } else {
    return false;
  }
}
