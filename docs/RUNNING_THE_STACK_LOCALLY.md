# Running the Stack Locally

This document provides additional information on running the System Initiative software stack locally.
Readers should first refer to the [README](../README.md) and [DEVELOPMENT_ENVIRONMENT](./DEVELOPMENT_ENVIRONMENT.md) before reading this document.

## Advanced Options

While the [README](../README.md) covers using `buck2 run dev:up`, there are two other ways to run the full stack locally:

- `buck2 run dev:up-standard`: run with `rustc` default build optimizations
- `buck2 run dev:up-debug`: run with `rustc` debug build optimizations

By default, the stack will run with `rustc` release build optimizations, which is what users and testers of the System Initiative software will want to use.
It runs the software in its intended state.
However, if you are a contributor seeking build times suitable for rapid iteration, you may want to use one of the aforementioned options.

_Warning: contributors should test your changes with integration tests and with release optimizations when running the full stack._
_The aforementioned options are solely recommended for rapid iteration._