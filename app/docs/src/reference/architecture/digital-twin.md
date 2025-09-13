---
outline: false
prev:
  text: 'Change Control'
  link: '/reference/architecture/change-control'
next:
  text: 'Function Execution Framework'
  link: '/reference/architecture/functions'
---

# Digital Twin

When modeling upstream resources for a provider, each resource maps 1:1 with a component in System Initiative, which lives on the snapshot as a subgraph.  Each component maintains dual state representations, two perspectives that coexist simultaneously, which establish the digital twin for a given provider’s resource:

__Intent:__ User or agent-declared configuration, expressed through property values and relationship definitions. Intent represents the desired system configuration independent of current reality. On the component, this is what’s stored in the domain tree.

__Reality:__ Provider reported configuration, resource status, and actual system state. Reality is populated by orchestrating periodic polling or on-demand queries. On the component, this is what’s stored in the resource_value tree.

## Drift

Drift between intent and reality is not hidden, but surfaced explicitly. For example, a virtual machine may record an intended size, while the provider reports a different size. Both perspectives coexist simultaneously, which reflects a fundamental truth of working with infrastructure: user intent and observable reality are not always aligned.

When there is a discrepancy between the intended configuration and the reported state, the engine records the difference. It can facilitate reconciliation or resolution with high precision via functions or user interaction, rather than blindly overwriting or abstracting it away. For example, users can decide whether to:

- __Accept Reality:__ Update intent to match the provider state
- __Enforce intent:__ Enqueue actions to align the provider state with intent
