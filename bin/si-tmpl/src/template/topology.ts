import type { TemplateContext } from "./context.ts";
import type {
  ComponentChange,
  CreateChange,
  PendingChanges,
  UpdateChange,
} from "./converge_types.ts";

/**
 * Orders changes based on dependencies using topological sort (Kahn's algorithm).
 *
 * This function ensures that components are created and updated in the correct order
 * to satisfy subscription dependencies. For example, if component A subscribes to
 * component B's attributes, B must be created before A.
 *
 * The algorithm:
 * 1. Builds a dependency graph from create/update dependencies
 * 2. Sorts creates by dependency order (components with no dependencies first)
 * 3. Sorts updates by dependency order (less critical but maintains consistency)
 * 4. Appends deletes last (nothing depends on deleted components)
 *
 * @param ctx - Template context for logging
 * @param pending - Pending changes with dependency information
 * @returns Ordered array of changes ready for execution
 * @throws Error if circular dependencies are detected
 */
export function topologicalSort(
  ctx: TemplateContext,
  pending: PendingChanges,
): ComponentChange[] {
  ctx.logger.trace("Computing dependency order");

  // Build dependency graph
  const graph = buildDependencyGraph(pending);

  // Sort creates by dependencies
  const sortedCreates = sortByDependencies(
    pending.creates,
    graph,
    ctx,
  );

  // Sort updates by dependencies (less critical but good practice)
  const sortedUpdates = sortByDependencies(
    pending.updates,
    graph,
    ctx,
  );

  // Deletes can happen last (nothing depends on deleted components)
  const allChanges: ComponentChange[] = [
    ...sortedCreates,
    ...sortedUpdates,
    ...pending.deletes,
  ];

  ctx.logger.trace("Ordered {count} changes", { count: allChanges.length });
  return allChanges;
}

interface DependencyGraph {
  edges: Map<string, Set<string>>; // workingSetId → set of dependencies
  inDegree: Map<string, number>; // workingSetId → number of dependencies
}

function buildDependencyGraph(pending: PendingChanges): DependencyGraph {
  const edges = new Map<string, Set<string>>();
  const inDegree = new Map<string, number>();

  // Initialize for all creates and updates
  const allChanges = [...pending.creates, ...pending.updates];
  for (const change of allChanges) {
    const wsId = change.workingSetComponent.id;
    edges.set(wsId, new Set(change.dependencies));
    inDegree.set(wsId, change.dependencies.length);
  }

  return { edges, inDegree };
}

function sortByDependencies<T extends CreateChange | UpdateChange>(
  changes: T[],
  graph: DependencyGraph,
  ctx: TemplateContext,
): T[] {
  // Kahn's algorithm for topological sort
  const sorted: T[] = [];
  const queue: T[] = [];
  const remaining = new Map<string, T>();

  // Initialize queue with changes that have no dependencies
  for (const change of changes) {
    const wsId = change.workingSetComponent.id;
    remaining.set(wsId, change);

    const deps = graph.inDegree.get(wsId) || 0;
    if (deps === 0) {
      queue.push(change);
    }
  }

  // Process queue
  while (queue.length > 0) {
    const change = queue.shift()!;
    sorted.push(change);

    const wsId = change.workingSetComponent.id;
    remaining.delete(wsId);

    // For each component that depends on this one
    for (const [dependentId, deps] of graph.edges.entries()) {
      if (deps.has(wsId)) {
        // Decrement in-degree
        const currentDegree = graph.inDegree.get(dependentId) || 0;
        graph.inDegree.set(dependentId, currentDegree - 1);

        // If no more dependencies, add to queue
        if (currentDegree - 1 === 0) {
          const dependentChange = remaining.get(dependentId);
          if (dependentChange) {
            queue.push(dependentChange);
          }
        }
      }
    }
  }

  // Check for cycles
  if (remaining.size > 0) {
    const cycleIds = Array.from(remaining.keys());
    ctx.logger.error(
      "Circular dependency detected in components: {ids}",
      { ids: cycleIds },
    );
    throw new Error(
      `Circular dependency detected among components: ${cycleIds.join(", ")}`,
    );
  }

  return sorted;
}
