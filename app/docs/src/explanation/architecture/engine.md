---
prev:
  text: 'Overview'
  link: '/explanation/architecture/index'
next:
  text: 'The Data Model'
  link: '/explanation/architecture/snapshot'
---

# The Distributed Execution Engine

The engine operates as the central orchestration layer, managing four core responsibilities across a distributed architecture.

## Mutation Processing

All system modifications flow through a structured mutation pipeline. Mutations are atomic operations that describe state transitions - for example, component creation, property updates, relationship changes, or schema modifications. The engine validates mutations against schema constraints, dependency requirements, and access control policies before applying them to the target snapshot and recording an audit trail for traceability.
The mutation processing subsystem maintains ordering guarantees within change sets while allowing concurrent operations across different change sets. This design enables multiple users or AI agents to work simultaneously without coordination overhead during the simulation phase.

## Change Control Enforcement

The engine implements change control through a branching model where the Head change set represents the canonical infrastructure state for the Workspace. All modifications occur within isolated change sets that can diverge arbitrarily from Head without affecting production systems.
Change set application requires explicit user or agent action, triggering a reconciliation process that replays all mutations against the current Head state. The engine handles conflicts through operational transforms, ensuring that concurrent modifications can be merged deterministically without manual conflict resolution.

## Reactive Change Propagation

When mutations occur, the engine calculates the minimal set of downstream functions that require re-execution. This dependency analysis traverses the DAG to identify affected components, their dependent functions, and any cross-component relationships that trigger cascading updates. The propagation system operates incrementally - as functions complete and their outputs change, additional downstream functions become eligible for execution. This approach minimizes unnecessary computation while ensuring that all dependent states are updated accordingly.

## Function Dispatch Management

The engine tracks, dispatches, and processes functions eligible for execution, considering their dependency requirements, resource constraints, and execution context.  As functions are dispatched, the engine ensures their results are reflected in the snapshot, and continually evaluates and dispatches newly eligible functions.
