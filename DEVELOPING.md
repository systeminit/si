# Developing

This document contains all information related to developing and running the SI stack.

## Supported Developer Environments

Environment | `x84_64 (amd64)` | `aarch64 (arm64)`
--- | --- | ---
Arch Linux | ✅ | 🚫
Fedora | ✅ | 🚫
macOS | ✅ | ✅
Ubuntu | ✅ | 🚫
WSL2 | ✅ | 🚫

We recommend using the latest stable Rust toolchain and latest LTS Node toolchain for your environment.
If unsure, the following tools are recommended to help manage your toolchains:

* [**rustup**](https://rustup.rs) 🦀: Rust, `cargo`, etc.
* [**volta**](https://volta.sh) ⚡: `node`, `npm`, etc.

### Preferred Environment Not Listed

If your preferred environment is not listed, please feel free to add it once the following conditions have been met:

1. It's been added to the (mostly) idempotent [bootstrap script](./scripts/bootstrap.sh)
2. The aforementioned script has been tested and remains (mostly) idempotent
3. Running the **Quickstart** steps below is successful and the UI is fully functional

_Please note:_ adding your preferred environment will also add you as a maintainer of its functionality throughout this repository.
If unsure where to start, you can look at a [PR from the past](https://github.com/systeminit/si/pull/589) to help.
If you are no longer using the environment, and you are the sole maintainer of the environment, you must remove it from the bootstrapper and the table above.

### Notes on aarch64 (arm64)

Few SI dependencies rely on using an `x86_64 (amd64)` host.
Fortunately, a compatibility layer, such as [Rosetta 2 on macOS](https://support.apple.com/en-us/HT211861) should suffice during builds.
You can install Rosetta 2 on macOS by executing the following:

```bash
softwareupdate --install-rosetta
```

Despite the above, if any dependency can be made to work on both `aarch64 (arm64)` and `x86_64 (amd64)`, we should do attempt to do so.
Not only is flexibility between architectures useful for local development, but it may also be useful in CI and/or production.

## Quickstart

The steps outlined in this guide can be used interchangeably, modified slightly, etc. depending on your
preferences and use cases.
However, for first time users, we recommend following this guide "as-is".

### Bootstrapping Your Environment (1/5)

For either running SI locally or developing SI, execute the following script:

```bash
./scripts/bootstrap.sh
```

The bootstrapper is (mostly) idempotent, so feel free to run it as many times as you like!
However, it _will_ upgrade existing packages without confirmations, so ensure that you are ready to do so.

### Checking Permissions and Authentication (2/5)

We need to ensure that we are [logged into Docker locally](https://docs.docker.com/engine/reference/commandline/login/)
and that the corresponding account can pull images from our [private repositories](https://hub.docker.com/orgs/systeminit/repositories).
Please reach out internally if your account cannot pull images from the private SI repositories.

### Checking for Potential Service Conflicts (3/5)

SI uses external services in conjunction with its native components.
These external services are deployed via `docker-compose` and are configured to stick to their default settings as closely as possible, including port settings.
Thus, it is worth checking if you are running these services to avoid conflicts when running SI.
Potentially conflicting services include, but are not limited to, the following:

* PostgreSQL DB
* OpenTelemetry
* NATS
* Watchtower

In the case of a port conflict, a good strategy is to temporarily disable the host service until SI is no longer being run.

### Running the SI Stack (4/5)

With all dependencies installed and required binaries in `PATH`, we are ready to go!
In one terminal pane (e.g. using a terminal multiplexer, such as `tmux`, or tabs/windows), execute the following:

```bash
make prepare
```

This will ensure that our database is running, our NATS server is running, the JS language server is built, and all crates are built.

Now, wait for the `postgres` database container to be running and ready to receive incoming client connection requests.
If it is not ready, `sdf` database migration will fail.

Once the database is ready, you can run `veritech`.

```bash
make veritech-run
```

In another terminal pane, run `sdf`.

```bash
make sdf-run
```

_Note:_ you can run `veritech` and `sdf` again without running the prepare target.

In a third terminal pane, execute the following command:

```bash
make app-run
```

This will run the web application, which you can access by navigating to https://localhost:8080.
Now, you have SI running!

### Tearing Down the SI Stack (5/5)

You can tear down SI and its external services by stopping the active `make` targets above and executing the following in the repository root:

```bash
make down
```

The above target will not only stop all running containers, but will remove them as well.

## Preparing Your Changes and Running Tests

Navigate to the `Makefile` in the [ci](./ci) directory to see local development
targets. These targets include code linting, formatting, running CI locally,
etc.

### Running All Lints

To verify that all lints will pass in CI, execute the following target:

```bash
( cd ci; make ci-lint )
```

> #### Using Optional Tidy Targets
>
> You can (optionally) use the "tidy" make targets before linting.
> Be careful, as the Rust-related tidy actions may perform more aggressive fixes than what the lint target checks for.
>
> ```bash
> ( cd ci; make tidy )
> ```

### Running Integration Tests

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
cargo test
```

### Debugging Integration Tests with Database Contents

Integration tests from the dal will not have their transactions committed and persisted to the database.
If you would like to view the state of the database via a debugger or after test conclusion, you will need to _temporarily_ refactor your test accordingly:

```rust
#[test]
async fn your_dal_integration_test(
    dal_context_builder: DalContextBuilder,
    jwt_secret_key: &JwtSecretKey,
    application_id: &NodeId,
) {
    let mut transactions_starter = dal_context_builder
        .transactions_starter()
        .await
        .expect("failed to build transactions starter");
    let transactions = transactions_starter
        .start()
        .await
        .expect("failed to start transactions");
    let (nba, _auth_token) = ::dal::test::helpers::billing_account_signup(
        &dal_context_builder,
        &transactions,
        jwt_secret_key,
    )
    .await;
    let ctx = ::dal::test::helpers::create_ctx_for_new_change_set_and_edit_session(
        &dal_context_builder,
        &transactions,
        &nba,
        *application_id,
    )
    .await;

    // Do your test stuff here!

    transactions.commit().await.expect("failed to commit");

    // After committing your transactions, you must end the test since you cannot use "ctx" again ("transactions" is
    // is consumed once you commit.
}
```

With these changes, you will be able to commit transactions and see them in the database.
However, please note: this refactor "hack" may produce unintended side effects that may or may not be relevant
(depending on what you are looking for).
It is also possible that the refactor example may not be entirely correct for your use case.

## Troubleshooting

If re-running the aforementioned [bootstrap script](./scripts/bootstrap.sh) does not solve your issue
and you are certain that `main` is stable, this section may help with troubleshooting and debugging.

### Wiping the Slate Clean

Having trouble running SI or its tests?
Want to go back to the beginning and wipe the slate clean?
We have a `make` target for that.

```bash
make troubleshoot
```

### Build and Runtime Errors on aarch64 (arm64)

For `aarch64 (arm64)` debugging, please refer to the aforementioned **Notes on aarch64 (arm64)** section.

### Hitting File Descriptor Limits in Integration Tests

Running all [dal integration tests](./lib/dal/tests/integration.rs) can result in hitting the file descriptor limit.
You may see NATS, Postgres and general failures containing the following message: "too many open files".
If this happens, you are likley hitting the file descriptors limit.

You can see all `ulimit` values by executing `ulimit -a`.
For your specific OS, please refer to its official documentation on how to increase the file descriptor limit
to a reasonable, stable, and likely-much-higher value.

> #### Setting the Limit with Persistence on macOS
>
> While we recommend referring to the official documentation when possible, sometimes, it does not
> exist!
> [This guide](https://becomethesolution.com/blogs/mac/increase-open-file-descriptor-limits-fix-too-many-open-files-errors-mac-os-x-10-14)
> from an unofficial source may help persist file descriptor limit changes on macOS.

## Reading and Writing Documentation

Our crates leverage `rustdoc` for seamless integration with `cargo doc`, [IntelliJ Rust](https://www.jetbrains.com/rust/),
[rust-analyzer](https://rust-analyzer.github.io/), and more.

### Reading Rust Documentation

Build the docs for all of our crates and open the docs in your browser at [dal](./lib/dal) by executing the following:

```bash
cargo doc --all
cargo doc -p dal --open
```

If you would like to live-recompile docs while making changes on your development branch, you can execute the following
make target:

```bash
# Choose one! Both work.
make doc
make docs
```

> Please note: [cargo-watch](https://github.com/watchexec/cargo-watch) needs to be installed before using the above make target.
>
> ```bash
> cargo install --locked cargo-watch
> ```

### Writing Rust Documentation

We try to follow the official ["How to write documentation"](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html) guide from `rustdoc` as closely as possible.
Older areas of the codebase may not follow the guide and conventions derived from it.
We encourage updating older documentation as whilst navigating through SI crates.

#### Additional Resources

* [RFC-1574](https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text): more API documentation conventions for `rust-lang`
* ["Making Useful Documentation Comments" from "The Book"](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments): a section of "The Book" covering useful documentation in the context of crate publishing
