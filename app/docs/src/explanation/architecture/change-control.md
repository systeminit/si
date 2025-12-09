---
outline:
  level: [2, 3, 4]
prev:
    text: 'The Data Model'
    link: '/explanation/architecture/snapshot'
next:
    text: 'Digital Twin'
    link: '/explanation/architecture/digital-twin'
---

# Change Control: Change Sets

System Initiative natively implements change control through the use of change sets, whose lifecycle is managed by the engine.  They can be thought of as similar to branches in git, with one Head change set representing the latest canonical state of your infrastructure (similar to the tip of main) and any number of open change sets with isolated pending changes, providing the simulation (feature branches that haven’t yet been merged).  A change set can diverge arbitrarily from the canonical state of Head, but its mutations remain isolated until explicitly applied.

## Change Set Lifecycle

- Creation: A new change set clones the current Head snapshot
- Mutation: All modifications occur within an isolated change set context
- Simulation: Reactive functions execute to maintain consistency within the simulation and provide feedback on proposed changes
- Application: Change set mutations are replayed against the current Head state, and subsequently replayed across all open change sets to ensure simulations are operating with the most up-to-date state
- Reconciliation: Operational transforms resolve conflicts and update all dependent change sets

## Operational Transform

Occasionally, concurrent mutations are made within a change set without knowledge of each other (two users edit the same component, in the same change set). More common, however, is that mutations are made across different change sets (without knowledge of each other), and both change sets need to be applied to Head. System Initiative uses [operational transforms](https://en.wikipedia.org/wiki/Operational_transformation) (OTs) rather than [conflict-free replicated data types](https://en.wikipedia.org/wiki/Conflict-free_replicated_data_type) (CRDTs) to handle concurrent modifications. This choice prioritizes real-time collaborative feedback over eventual convergence across distributed replicas.

The OT system transforms operations based on the concurrent operations that preceded them. In cases where multiple changes occur simultaneously without knowledge of each other, the engine determines the appropriate transformation to maintain semantic consistency and preserve user intent when processing both changes.

## Head Change Set Constraints

The Head change set operates under three unique constraints:

### Action Exclusivity

Actions intended to mutate upstream resources are only ever dispatched from the Head change set. This ensures that all infrastructure changes flow through the change control process, preventing unauthorized modifications without appropriate review.  

### Restricted Mutation

Similar to branch protections in git, the Head change set prevents direct mutation without the use of change sets, with the following exceptions:

1. Refresh Actions: Query third-party providers and update the model's representation of reality
2. Qualifications: Validate the model to test compliance with organizational policy or the correctness/validity of a component's configuration
3. Action State Change: Actions that have been queued in a change set, approved, and applied to Head for execution, can be retried if they fail, can be placed on hold if they haven't yet been dispatched, and can be requeued if on hold.

### Universal Replay

All mutations applied to Head are replayed on every open change set. This keeps simulations current with reality, ensuring that proposed changes are continually validated against the latest infrastructure state.
