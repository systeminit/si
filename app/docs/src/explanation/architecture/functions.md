---
outline:
  level: [2, 3, 4]
prev:
  text: 'Digital Twin'
  link: '/explanation/architecture/digital-twin'
next:
  text: 'Tenancy and Access Control'
  link: '/explanation/architecture/tenancy'
---


# Function Execution Framework

Functions are one of the most important primitives within the System Initiative architecture and are similarly represented by a subgraph of the snapshot. As mutations flow, the engine is responsible for resolving dependencies and dispatching functions as soon as they are eligible to run.

## Function Categories and Dispatch Behavior

Function categories determine their triggering behavior.

### Reactive Functions

 Automatically dispatched when their input dependencies change. These functions maintain derived state, validate configurations, and compute dependent values. Reactive functions execute within the same change set context where their dependencies were modified and include code generation functions, qualifications, transformation functions, authentication functions, and attribute functions.

### Ad Hoc Functions

 Explicitly invoked by users, AI agents, or integrations. These include management operations as well as qualifications (in addition to being reactive). Ad hoc functions can modify state within the context of the change set where the function was invoked.

### Queued Functions (Actions)

Execute only against the Head change set and interact with external APIs to modify or interrogate external infrastructure, resulting in updates to the given component's resource (the reality side of the [digital twin](./digital-twin.md)). This includes resource creation, modification, and deletion operations, as well as reading external state and updating the model accordingly. Intent to run an action is recorded on the snapshot, queued within a change set, and reviewed through the change control process. When applied, the engine calculates the correct ordering for actions based on the dependencies of the component being acted on and dispatches it accordingly.

## Dependency Resolution

The engine maintains a dependency graph for all functions, enabling efficient dispatch ordering and maximal parallelization. When mutations occur, the engine:

1. Identifies all functions with modified inputs or newly established intent to execute
2. Builds a dependency graph of function inputs
3. Uses the graph to dispatch functions in parallel as their dependencies become satisfied
4. Records function results, subsequently used in downstream functions

This approach ensures that functions execute in the correct order while maximizing the parallelization opportunities.

## Function Execution Security

Functions execute in isolated [Firecracker](https://firecracker-microvm.github.io/) VMs with:

- On-demand provisioning: New VMs are spawned for each function execution
- Resource Limits: CPU, memory, and execution time constraints
- Network Isolation: Controlled access to external APIs and services
- Just-in-Time Credential Access: Necessary credentials are only provided to functions during execution

The isolation model prevents functions from interfering with each other while providing secure access to external resources through the credential management system.
