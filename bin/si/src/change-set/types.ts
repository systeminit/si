import type { GlobalOptions } from "../cli.ts";

export interface ChangeSetCreateOptions extends GlobalOptions {
  name: string;
}

export interface ChangeSetAbandonOptions extends GlobalOptions {
  changeSetIdOrName: string;
}

export interface ChangeSetListOptions extends GlobalOptions {
  output?: string;
}
