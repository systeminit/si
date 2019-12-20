# System Graph

```
graph
├── editor
│   └── node
│       ├── @types
│       ├── components
│       └── operations
├── providers
│   └── aws
│       ├── operators
│       └── resources
└── viewers
    └── schematic
```

## Graph Editor

Core graph authoring.

## Graph Providers

Provider specific implementation.

## Graph Viewers

Different ways to vizualize a graph.

## UX Experience.

### Graph Editor

A user author a system graph using the graph editor. When done the use click _save_ to save a new version. A window ask the user for comments and the new version is saved. This is basically a commit in a repo for this system graph. The graph file can also be pulled via the CLI.

### Graph Schematic

A user review a system schematic in order to better undertand the system components and how they are wired together.

# Implementation

We should build a js code generator from yaml to build nodes library.
