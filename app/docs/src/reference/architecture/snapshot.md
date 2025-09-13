---
outline:
  level: [2, 3, 4]
prev:
  text: 'The Distributed Execution Engine'
  link: '/reference/architecture/engine'
next:
  text: 'Change Control'
  link: '/reference/architecture/change-control'
---

# The Data Model: Immutable Snapshots

The workspace snapshot, an immutable directed acyclic graph ([DAG](https://en.wikipedia.org/wiki/Directed_acyclic_graph)), is the core data type in System Initiative and represents the complete system state at a specific point in time.

## Snapshot Structure

The structure begins with a single root node and a collection of direct neighbors that each represent a different category of primitives within System Initiative. Every node in the snapshot is connected indirectly to the root through the categories.

### Nodes

There are several distinct node types in the snapshot, including but not limited to:

- __Modules:__ Portable collections of schemas and functions
- __Schemas:__ Component type definitions with property schemas, validation rules, and function definitions
- __Functions:__ Executable code units with dependency declarations and execution metadata
- __Components:__ An instance of a schema, with all configuration, relationships, and resource tracking if applicable
- __Values:__ Configuration data, inputs to functions, computed results, and provider responses
- __Views:__ Partition containing a set of components
- __Secrets:__ Reference pointer to an encrypted credential
- __Actions:__ Intended operations for external system modification or interaction

Nodes contain metadata, pointers to content-addressable storage (CAS) objects, and a Merkle-tree hash of all downstream nodes and edges.  This design enables:

- Deduplication of identical content across snapshots
- Atomic updates to content through pointer swapping
- Efficient node differentiation through Merkle-tree comparison

### Edges

Edges encode multiple relationship types that enable deterministic graph traversals, for example:

- Containment: Hierarchical ownership (Ex: Views contain components)
- Dependency: Execution ordering constraints (Ex: Function A depends on the output of function B)
- Lineage: Definition tracking and versioning (Ex: Component was created from Schema Variant A)

## Versioning and Copy-on-Write Semantics

Versioning of snapshots is explicit through the use of [change sets](./change-control.md). During a mutation, the source snapshot establishes a baseline, and all nodes/edges are cloned using copy-on-write semantics - retaining any unchanged pointers to relevant CAS objects. This approach enables rapid snapshot creation and efficient storage utilization while maintaining immutability guarantees. The Merkle-tree hash allows fast snapshot comparison, enabling the engine to compute minimal mutation sets when synchronizing or replaying changes.
