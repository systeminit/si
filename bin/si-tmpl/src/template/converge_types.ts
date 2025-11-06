import type { TemplateComponent } from "./context.ts";

/**
 * Component from SI with template tags
 */
export interface ExistingSetComponent {
  id: string;
  schemaId: string;
  name: string;
  resourceId: string;
  // deno-lint-ignore no-explicit-any
  attributes: { [key: string]: any };
  templateWorkingSetId: string; // from /si/tags/templateWorkingSetId
}

/**
 * Subscription source specification
 */
export interface SubscriptionSource {
  component: string; // component ID or name
  path: string; // attribute path
  func?: string; // optional transformation function
}

/**
 * Attribute-level diff for updates
 */
export interface AttributeDiff {
  // deno-lint-ignore no-explicit-any
  set: Map<string, any>; // path → new value
  unset: string[]; // paths to remove
  subscriptions: Map<string, SubscriptionSource>; // path → subscription
}

/**
 * Create change operation
 */
export interface CreateChange {
  type: "create";
  workingSetComponent: TemplateComponent;
  // deno-lint-ignore no-explicit-any
  attributes: { [key: string]: any };
  dependencies: string[]; // workingSet IDs this depends on
}

/**
 * Update change operation
 */
export interface UpdateChange {
  type: "update";
  existingComponent: ExistingSetComponent;
  workingSetComponent: TemplateComponent;
  attributeDiff: AttributeDiff;
  nameChange?: { from: string; to: string };
  dependencies: string[];
}

/**
 * Delete change operation
 */
export interface DeleteChange {
  type: "delete";
  existingComponent: ExistingSetComponent;
}

/**
 * Union type of all change operations
 */
export type ComponentChange = CreateChange | UpdateChange | DeleteChange;

/**
 * Container for all pending changes
 */
export interface PendingChanges {
  creates: CreateChange[];
  updates: UpdateChange[];
  deletes: DeleteChange[];
  workingSetById: Map<string, TemplateComponent>;
  existingByWorkingSetId: Map<string, ExistingSetComponent>;
  existingByDynamicName: Map<string, ExistingSetComponent>;
}
