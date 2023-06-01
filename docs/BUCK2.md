# `buck2`

This document contains information on using [`buck2`](https://github.com/facebook/buck2) within this repository.
Using `buck2` is currently experimental and in active development.

## Quickstart

Here are helpful commands:

```bash
buck2 run :prepare
buck2 run :council
buck2 run :veritech
buck2 run :pinga
buck2 run :sdf
buck2 run :web
buck2 run :down
```

You may need to vendor crates when building third party crates for the first time and you may need
to re-vendor crates when building third party Rust crates results in failure.
In either case, run (or re-run) the following command:

```bash
reindeer --third-party-dir third-party/rust vendor
```