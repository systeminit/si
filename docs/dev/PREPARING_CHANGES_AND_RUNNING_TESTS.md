# Preparing Changes and Running Tests

This document contains information related to preparing your changes and running tests.

## Running Checks

You can check a component with:

```bash
make check//bin/veritech
```

Where `bin/veritech` is the path to the component you want to check.

## Running All Checks

You can run all the checks with:

```bash
make check
```

> ### Using Optional Fix Targets
>
> You can (optionally) use the `fix` make targets before running checks.
> Be careful, as the Rust-related fix actions may perform more aggressive fixes than what the check target checks for.
>
> ```bash
> make fix
> ```

## Running Tests

You can run tests for components with:

```bash
make prepare
```

Followed by:

```bash
make test//bin/veritech
```

Where `bin/veritech` is the path to the component you want to test. If you want the environment to be
cleaned and started automatically (blowing away the data in the database) run this instead:

```bash
make FORCE=true test//lib/sdf-server
```

## Running the CI tests locally

To ensure your code will pass CI, you can run the exact same code that the CI servers themselves will run.

```bash
CI=true make down ci
```

This will evaluate the delta between your current branch and `main`, and run only the tests and checks
that are relevant to your changes.

_However_, you should rarely need to run the above commands.
It is usually best to run individual tests or module(s) of tests (likely from
the [`dal integration tests directory`](./lib/dal/tests/integration_test/))
that cover code you have edited.

## Running Integration Tests Manually

You can also run individual [dal](./lib/dal) integration tests before bringing
up the entire SI stack, as needed. This can be done in the root of the
repository with two terminal panes.

In one pane, prepare the test environment:

```bash
make prepare
```

In the other pane, run your test(s):

```bash
# Running one test with backtrace enabled
RUST_BACKTRACE=1 cargo test <your-individual-test-name>

# Running all tests within "integration_test" with backtrace set to "full"
RUST_BACKTRACE=full cargo test integration_test

# Running all tests
make test
```

If you would like to log output during tests with timestamps, use `tracing::info` and add the following environment
variable.

```bash
SI_TEST_LOG=info
```

### Want To See The Live Log Stream?

If you'd like to see a live log stream during test execution, use the `SI_TEST_LOG` variable in conjunction with
the `--nocapture` flag for `cargo test`.

```shell
SI_TEST_LOG=info cargo test -p dal --test integration <your-test> -- --nocapture
```

You can combine this with the `DEBUG` environment variable for `lang-js` for even more information.

```shell
DEBUG=* SI_TEST_LOG=info cargo test -p dal --test integration <your-test> -- --nocapture
```

### Migrations Running Too Slow? Try Disabling or Choosing Builtin Schema Migrations!

If your integration test does not rely on builtin `Schema(s)` and `SchemaVariant(s)` (e.g. "Docker Image" and
"AWS EC2"), you can disable migrating them by passing in `"none"` or `"false"` (or some variant of them) to the
appropriate environment variable.

```shell
SI_TEST_BUILTIN_SCHEMAS=none cargo test -p dal --test integration <your-test>
```

You can also choose individual builtins to migrate with a comma-separated list.

```shell
SI_TEST_BUILTIN_SCHEMAS=Schema One,Schema Two cargo test -p dal --test integration <your-test>
```

> Note: you may not want to wrap your list in `"` characters, depending on your development environment as they may
> be unintentionally included in the list item(s).

If you want migrations to run as they would by default, remove the environment variable or set it to `"all"` or `"true"`
(or some variant of them).

```shell
SI_TEST_BUILTIN_SCHEMAS=all cargo test -p dal --test integration <your-test>
```

## Debugging Integration Tests with Database Contents

Integration tests in the `lib/dal` crate typically perform their work without
committing and persisting to the database (that is, the transaction in each
test is rolled back when the transaction is dropped). However, if you would
like to view the state of the database after the test finishes running, you can
modify your test accordingly:

```rust
// Note, you'll want to own your `DalContext` so that you can consume it
// when commiting. Let's call it `octx` for "owned `ctx`".
#[test]
async fn your_dal_integration_test(octx: DalContext) {
    // Save the owned context for later.
    let ctx = &octx;

    // Perform your test work, mutate the `ctx`, etc.

    // Construct a new ctx, resuing the connected `Connections` with identical
    // tenancies, visibility, and history actor
    let ctx = commit_and_continue(octx).await;

    // More test code, if applicable.
}
```

With these changes, you will be able to commit transactions and see them in the
database.
