import type { TemplateComponent } from "./context.ts";

/**
 * Represents a component that exists in System Initiative and was created by this template.
 *
 * ExistingSetComponents are identified by the `/si/tags/templateWorkingSetId` tag which
 * links them back to the working set component ID that created them. This enables
 * idempotent template execution by matching existing components to working set components.
 */
export interface ExistingSetComponent {
  /** Unique component ID (ULID) assigned by System Initiative */
  id: string;
  /** Schema ID defining the component type */
  schemaId: string;
  /** Schema name (human-readable) */
  schemaName?: string;
  /** Display name of the component */
  name: string;
  /** Resource ID if the component has been materialized to a real infrastructure resource */
  resourceId: string;
  /** Component attributes (all paths, not filtered like TemplateComponent) */
  // deno-lint-ignore no-explicit-any
  attributes: { [key: string]: any };
  /** Working set component ID that created this component (from /si/tags/templateWorkingSetId) */
  templateWorkingSetId: string;
}

/**
 * Specification for a subscription source component and attribute path.
 *
 * Subscriptions allow one component to receive values from another component's attributes.
 * This interface defines the resolved subscription target (after search or name resolution).
 */
export interface SubscriptionSource {
  /** Component ID (ULID) of the source component */
  component: string;
  /** Attribute path on the source component (e.g., "/domain/connectionString") */
  path: string;
  /** Optional transformation function to apply to the subscribed value (e.g., "si:normalizeToArray") */
  func?: string;
}

/**
 * Represents the differences between a working set component and an existing component.
 *
 * Used to compute the minimal set of changes needed to update an existing component
 * to match the desired state in the working set. Changes are categorized as sets,
 * unsets, and subscription updates.
 */
export interface AttributeDiff {
  /** Map of attribute paths to new values that should be set */
  // deno-lint-ignore no-explicit-any
  set: Map<string, any>;
  /** Array of attribute paths that should be removed */
  unset: string[];
  /** Map of attribute paths to subscription sources that should be set */
  subscriptions: Map<string, SubscriptionSource>;
}

/**
 * Represents a pending operation to create a new component in System Initiative.
 *
 * Create changes are generated for working set components that don't match any
 * existing component. The new component will be tagged with the working set ID
 * for future idempotent matching.
 */
export interface CreateChange {
  /** Change type discriminator */
  type: "create";
  /** The working set component to create */
  workingSetComponent: TemplateComponent;
  /** Initial attributes to set on the new component */
  // deno-lint-ignore no-explicit-any
  attributes: { [key: string]: any };
  /** Array of working set component IDs this create depends on (for topological sorting) */
  dependencies: string[];
}

/**
 * Represents a pending operation to update an existing component in System Initiative.
 *
 * Update changes are generated when a working set component matches an existing
 * component but has different attributes or name. Only the differing attributes
 * are included in the attributeDiff.
 */
export interface UpdateChange {
  /** Change type discriminator */
  type: "update";
  /** The existing component to update */
  existingComponent: ExistingSetComponent;
  /** The working set component with desired state */
  workingSetComponent: TemplateComponent;
  /** Attribute-level differences to apply */
  attributeDiff: AttributeDiff;
  /** Optional name change if the component is being renamed */
  nameChange?: { from: string; to: string };
  /** Array of working set component IDs this update depends on (for topological sorting) */
  dependencies: string[];
}

/**
 * Represents a pending operation to delete an existing component in System Initiative.
 *
 * Delete changes are generated for existing components that were created by this
 * template but no longer exist in the working set. This cleans up components
 * that were removed from the template.
 */
export interface DeleteChange {
  /** Change type discriminator */
  type: "delete";
  /** The existing component to delete */
  existingComponent: ExistingSetComponent;
}

/**
 * Union type of all possible component change operations.
 *
 * Used throughout the convergence pipeline to represent pending changes
 * that need to be applied to System Initiative.
 */
export type ComponentChange = CreateChange | UpdateChange | DeleteChange;

/**
 * Container for all pending changes and lookup maps used during convergence.
 *
 * This structure organizes changes by type (create/update/delete) and provides
 * efficient lookup maps for matching working set components to existing components.
 */
export interface PendingChanges {
  /** Array of components to create */
  creates: CreateChange[];
  /** Array of components to update */
  updates: UpdateChange[];
  /** Array of components to delete */
  deletes: DeleteChange[];
  /** Map of working set component ID to TemplateComponent for fast lookup */
  workingSetById: Map<string, TemplateComponent>;
  /** Map of working set ID to matching ExistingSetComponent (from templateWorkingSetId tag) */
  existingByWorkingSetId: Map<string, ExistingSetComponent>;
  /** Map of dynamic name to ExistingSetComponent (from templateDynamicName tag) for copied components */
  existingByDynamicName: Map<string, ExistingSetComponent>;
}
