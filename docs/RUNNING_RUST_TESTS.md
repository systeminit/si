# Running Rust Tests

This document contains information related running Rust tests.

## Setup

Before running Rust based tests, we should ensure that dependent services are running.

```bash
buck2 run dev:platform
```

_Please note: not all tests require dependent services (e.g. unit tests for a small library), but if you are unsure,
we recommend running dependent services in advance._

## Running All Tests

For a given crate, you can use `buck2 test`.

```bash
buck2 test <crate>:test
```

To see all options, use `buck2 targets` with a `:` character at the end.

```bash
buck2 targets <crate>:
```

## Running Individual Tests

You can also run individual tests, as needed.
Instead of using `buck2 test`, we will use `buck2 run`.

Here is an example with an individual [dal](../lib/dal) integration test:

```bash
buck2 run //lib/dal:test-integration -- edge::new
```

Here is the same test, but with a precise pattern using the `--exact` flag.

```bash
buck2 run //lib/dal:test-integration -- --test integration integration_test::internal::edge::new -- --exact
```

Let's say the test has been ignored with "`#[ignore]`" and we would like to run it.
We will use the `--ignored` flag.

```bash
buck2 run //lib/dal:test-integration -- edge::new -- --ignored
```

If you'd like to see STDOUT rather than it being captured by the test executor, use the `--nocapture` flag.

```bash
buck2 run //lib/dal:test-integration -- edge::new -- --nocapture
```

## Using Environment Variables

You can prefix your `buck2 run` executions with environment variables of choice.

Here is an example for running one test within a crate and backtrace enabled:

```bash
RUST_BACKTRACE=1 buck2 run <crate>:test-integration -- <pattern>
```

Let's say you wanted to run all tests, but set backtrace to full.
You can do that too.

```bash
RUST_BACKTRACE=full buck2 run <crate>:test-integration
```

### Want To See The Live Log Stream?

If you'd like to see a live log stream during test execution, use the `SI_TEST_LOG` variable in conjunction with
the `--nocapture` flag.

```shell
SI_TEST_LOG=info buck2 run <crate>:test-integration -- <pattern> -- --nocapture
```

You can combine this with the `DEBUG` environment variable for `lang-js` for even more information.

```shell
DEBUG=* SI_TEST_LOG=info buck2 run <crate>:test-integration -- <pattern> -- --nocapture
```

### Migrations Running Too Slow? Try Disabling or Choosing Builtin Schema Migrations!

If your integration test does not rely on builtin `Schema(s)` and `SchemaVariant(s)` (e.g. "Docker Image" and
"AWS EC2"), you can disable migrating them by passing in `"none"` or `"false"` (or some variant of them) to the
appropriate environment variable.

```shell
SI_TEST_BUILTIN_SCHEMAS=none buck2 run <crate>:test-integration -- <pattern>
```

You can also choose individual builtins to migrate with a comma-separated list.

```shell
SI_TEST_BUILTIN_SCHEMAS=Schema One,Schema Two buck2 run <crate>:test-integration -- <pattern>
```

> Note: you may not want to wrap your list in `"` characters, depending on your development environment as they may
> be unintentionally included in the list item(s).

If you want migrations to run as they would by default, remove the environment variable or set it to `"all"` or `"true"`
(or some variant of them).

```shell
SI_TEST_BUILTIN_SCHEMAS=all buck2 run <crate>:test-integration -- <pattern>
```
