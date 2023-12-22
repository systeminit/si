# System Initiative (New Engine)

This document serves to help developers and early users of the new engine.
See [pull request #2879](https://github.com/systeminit/si/pull/2879) for more information.

## Disclaimer

All guides in this document assume your local development environment is ready to develop and run the System Initiative software.
In addition, the new engine is not ready for use and may result in loss of data.

## Preparing Your Changes

This guide provides the minimum steps for preparing changes for the new engine.

1. `cargo check --all-targets --all-features`
1. `cargo fmt`
1. `cargo clippy --all-targets --all-features`
1. `buck2 run //lib/dal:test-unit -- workspace_snapshot::graph`
1. `buck2 run //lib/dal:test-integration -- new_engine`
1. (small changes) `buck2 build //lib/...` and build relevant `//bin/` targets
1. (big changes) `buck2 run dev:up` and test the application by hand

## Running the Stack

Fortunately, running the stack works the same way it does on `main`.

```sh
buck2 run dev:up
```
