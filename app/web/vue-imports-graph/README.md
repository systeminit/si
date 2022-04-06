# Vue Imports Graph

This CLI application generates a dependency tree of local imports between Vue files.
It is experimental and currently uses string parsing, so results may be inaccurate.

## Usage

In any directory within SI, execute the following:

```shell
cargo run --bin vue-imports-graph
```

This will perform traversal from `app/web/src`.