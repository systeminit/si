# System Initiative (New Engine)

This document serves to help developers and early users of the new engine.
See [pull request #3113](https://github.com/systeminit/si/pull/3113) for more information.

## Disclaimer

All guides in this document assume your local development environment is ready to develop and run the System Initiative software.
In addition, the new engine is not ready for use and may result in loss of data.

## Preparing Your Environment

Preparing your environment works the exact same way it does on `main`.
_However,_ you need the latest images for supporting services in order to work with the new engine.
This is mainly because NATS requires Jetsream to be enabled, which is not the case in older "systeminit/nats" images.

Run the following command to get your images up to date:

```bash
buck2 run dev:pull
```

## Preparing Your Changes

This guide provides the minimum steps for preparing changes for the new engine.

1. `cargo check --all-targets --all-features`
1. `cargo fmt`
1. `cargo clippy --all-targets --all-features`
1. `buck2 run //lib/dal:test-unit`
1. `buck2 run //lib/dal:test-integration` _(side note: bring back tests as they become relevant again)_
1. `buck2 build //lib/... //bin/pinga //bin/sdf //bin/rebaser //bin/veritech //bin/council`
1. `buck2 run dev:up` and test the application by hand

## Running the Stack

Fortunately, running the stack works the same way it does on `main`.

```sh
buck2 run dev:up
```
