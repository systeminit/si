import type { GlobalOptions } from "../cli.ts";

export interface ChangeSetCreateOptions extends GlobalOptions {
  name: string;
  open?: boolean;
}

export interface ChangeSetListOptions extends GlobalOptions {
  output?: string;
}

/** Base interface for change set commands that operate on a specific change set */
export interface ChangeSetByIdOrNameOptions extends GlobalOptions {
  changeSetIdOrName: string;
}

export type ChangeSetAbandonOptions = ChangeSetByIdOrNameOptions;

export type ChangeSetOpenOptions = ChangeSetByIdOrNameOptions;

export interface ChangeSetApplyOptions extends ChangeSetByIdOrNameOptions {
  /** Don't wait for actions to complete, return immediately after applying */
  detach?: boolean;
}

export interface ChangeSetReviewOptions extends ChangeSetByIdOrNameOptions {
  /** Include resource code diffs (CloudFormation/Terraform) */
  includeResourceDiff?: boolean;
}
