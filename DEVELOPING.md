# Developing

This document contains information related to developing and running the SI stack.
However, since it cannot fit everything, you can check out the contents of the [docs directory](./docs) for even more
information.

## Table of Contents

- [Supported Developer Environments](#supported-developer-environments)
- [Quickstart](#quickstart)
- [Learning About SI Concepts](#learning-about-si-concepts)
- [Repository Structure](#repository-structure)
- [Preparing Your Changes and Running Tests](#preparing-your-changes-and-running-tests)

## Supported Developer Environments

| Environment | `x84_64 (amd64)` | `aarch64 (arm64)` |
|-------------|------------------|-------------------|
| Arch Linux  | ✅               | 🚫                |
| Fedora      | ✅               | 🚫                |
| macOS       | ✅               | ✅                |
| Pop!_OS     | ✅               | 🚫                |
| Ubuntu      | ✅               | 🚫                |
| WSL2        | ✅               | 🚫                |

We recommend using the latest stable Rust toolchain and latest LTS Node toolchain for your environment.
If unsure, the following tools are recommended to help manage your toolchains:

* [**rustup**](https://rustup.rs) 🦀: Rust, `cargo`, etc.
* [**volta**](https://volta.sh) ⚡: `node`, `npm`, etc.

### Preferred Environment Not Listed

If your preferred environment is not listed, please feel free to add it once the following conditions have been met:

1. It's been added to the (mostly) idempotent [bootstrap script](./scripts/bootstrap.sh)
2. The aforementioned script has been tested and remains (mostly) idempotent
3. Running the **Quickstart** steps below is successful and the UI is fully functional

_Please note:_ adding your preferred environment will also add you as a maintainer of its functionality throughout this
repository.
If unsure where to start, you can look at a [PR from the past](https://github.com/systeminit/si/pull/589) to help.
If you are no longer using the environment, and you are the sole maintainer of the environment, you must remove it from
the bootstrapper and the table above.

### Notes on aarch64 (arm64)

Few SI dependencies rely on using an `x86_64 (amd64)` host.
Fortunately, a compatibility layer, such as [Rosetta 2 on macOS](https://support.apple.com/en-us/HT211861) should
suffice during builds.
You can install Rosetta 2 on macOS by executing the following:

```bash
softwareupdate --install-rosetta
```

Despite the above, if any dependency can be made to work on both `aarch64 (arm64)` and `x86_64 (amd64)`, we should
attempt to do so.
Not only is flexibility between architectures useful for local development, but it may also be useful in CI and/or
production.

## Quickstart

The steps outlined in this guide can be used interchangeably, modified slightly, etc. depending on your
preferences and use cases.
However, for first time users, we recommend following this guide "as-is".

### Bootstrapping Your Environment (1/5)

First, ensure that Docker is installed on your machine and the `docker` executable is in your `PATH`. Then, for either
running SI locally or developing SI, execute the following script:

```bash
./scripts/bootstrap.sh
```

The bootstrapper is (mostly) idempotent, so feel free to run it as many times as you like!
However, it _will_ upgrade existing packages without confirmations, so ensure that you are ready to do so.

### Checking Permissions and Authentication (2/5)

We need to ensure that we are [logged into Docker locally](https://docs.docker.com/engine/reference/commandline/login/)
and that the corresponding account can pull images from
our [private repositories](https://hub.docker.com/orgs/systeminit/repositories).
Please reach out internally if your account cannot pull images from the private SI repositories.

You should also configure your aws cli to use the SI account:

```bash
aws configure
```

passing the following responses to each prompt:

- AWS Access Key ID: The `Access Key` field on
  this [1password entry](https://start.1password.com/open/i?a=6FRDDOEI5JBKHJJAMQIKAEFWD4&v=y5uwcpkwsqeppqg4cwkxnnpwdm&i=mw3mygbdcd66pgn4hgkroicssi&h=systeminitiativeinc.1password.com)
- AWS Secret Access Key:  The `Secret Key` field on the
  same [1password entry](https://start.1password.com/open/i?a=6FRDDOEI5JBKHJJAMQIKAEFWD4&v=y5uwcpkwsqeppqg4cwkxnnpwdm&i=mw3mygbdcd66pgn4hgkroicssi&h=systeminitiativeinc.1password.com)
  as above
- Default region name: `us-east-2`
- Default output format: Leave empty

### Checking for Potential Service Conflicts (3/5)

SI uses external services in conjunction with its native components.
These external services are deployed via [`docker compose`](https://docs.docker.com/compose/) and are configured to stick to their default settings as
closely as possible, including port settings.
Thus, it is worth checking if you are running these services to avoid conflicts when running SI.
Potentially conflicting services include, but are not limited to, the following:

* PostgreSQL DB
* OpenTelemetry
* NATS
* Faktory
* Watchtower

In the case of a port conflict, a good strategy is to temporarily disable the host service until SI is no longer being
run.

### Running the SI Stack (4/5)

With all dependencies installed and required binaries in `PATH`, we are ready to go!
In one terminal pane (e.g. using a terminal multiplexer, such as `tmux`, or tabs/windows), execute the following:

```bash
make prepare
```

If you would like to skip pulling potentially newers Docker images before running the supporting services, then set `SI_SKIP_PULL` to a non-empty value, for example:

```bash
make prepare SI_SKIP_PULL=true
```

> #### Running Postgres on macOS `aarch64` (i.e. `arm64` or "Apple Silicon")
>
> As of October 2022, Docker Desktop runs Linux `x86_64` (i.e. `amd64`) images in a `qemu`, Linux `x86_64` VM.
> When using the official Postgres `x86_64` Docker images, you may notice significant performance
> degradation compared to users of Linux `amd64` machines, in particular.
>
> To avoid the issue, run the preparation target with the following variable set:
>
> ```bash
> make LOCAL_PG=true prepare
> ```
>
> This will run a local instance of Postgres that will be performant for your hardware.

This will ensure that our database is running, our NATS server is running, and faktory is running.

Now, wait for the `postgres` database container to be running and ready to receive incoming client connection requests.
If it is not ready, `sdf` database migration will fail.

Once the database is ready, we start running the "homemade" components of our stack.

> **Before We Start: How Will I Know That Each Component Is Ready?**
>
> For backend services like `veritech` and `sdf`, there will usually be an `INFO`-level log indicating that the
> webserver has bound to a port and is ready to receive messages.
> This may be subject to change (e.g. underlying library is upgraded to a new major version and the startup sequence
> changes) and will vary from component to component.

First, we run `veritech`.

```bash
make run//bin/veritech
```

In another terminal pane, run `pinga`.

```bash
make run//bin/pinga
```

In another terminal pane, run `council`.

```bash
make run//bin/council
```

In another terminal pane, run `sdf`.

```bash
make run//bin/sdf
```

In a final terminal pane, execute the following command:

```bash
make run//app/web
```

This will run the web application, which you can access by navigating to http://localhost:8080.
Now, you have SI running!


> **Using CLion Run Configurations Instead of Terminal Panes**
>
> This repository contains CLion run configurations for most of these targets, in addition to a `Run SI` compound target
> for running all the targets at once. They should be listed on the run menu automatically and are called
> `Prepare`, `Run [SDF | Veritech | Pinga | Web]` and `Teardown` (which is related to the next topic).
>
> Using them you should be able to run the whole stack via CLion's Run tool window instead of using multiple shells!

### Tearing Down the SI Stack (5/5)

You can tear down SI and its external services by stopping the active `make` targets above and executing the following
in the repository root:

```bash
make down
```

The above target will not only stop all running containers, but will remove them as well.

## Learning About SI Concepts

As referenced in [CODE_DOCUMENTATION](./docs/dev/CODE_DOCUMENTATION.md), the `rustdoc` static webpages are an entrypoint
into learning about the Rust modules and structs that back many SI concepts.

Let's say you want to learn about what a `Component` is.
You can generate and open the Rust documentation locally via the following command:

```bash
cargo doc --open -p dal
```

Moreover, there are resources in [docs](./docs), [designs](./designs), our Miro boards, and our Figma projects that
may prove useful as well.

## Repository Structure

While there are other directories in the project, these are primarily where
most of the interesting source code lives and how they are generally organized:

| Directory    | Description |
|--------------|-------------|
| `app/`       | Components that build web front ends, GUIs, or other desktop applications |
| `bin/`       | Components that are intended to produce a program, CLI, server, etc. |
| `component/` | Components that tend to produce primarily Docker container images and other ancillary tooling |
| `lib/`       | Components that are supporting libraries and packages |
| `mk/`        | Common Makefile targets and shared make behaviors |

### Makefile

A Makefile driven by the `make` program constitutes the primary build, test,
and release system for this project. While not perfect and not unique to
solving this class of problem, as they say "it gets the job done".

#### Base Makefile Targets

The `Makefile` at the root of the project is responsible for providing high
level targets that will apply to all child components of the project as well as
specialized tasks as required by the continuous integration system (CI) and our
deployment and delivery system. Lastly, it provides common verb-prefixed tasks
that will trigger tasks for each component where relevant.

The following is a set of make targets provided by the base Makefile. A
shortened summary of these is available by running `make help` at the root of
the project.

In tasks where `<cmpt>` is used, this is short for "component" and refers
specifically to the path to the component. For example, to build the SDF server
binary which can be found in `bin/sdf`, you can run: `make build//bin/sdf`.
Similarly, to run the test suite for `lib/si-data` while skipping pre-test
dependencies you can run: `make test//lib/si-data//TEST`.

| Target                 | Description |
|------------------------|-------------|
| `help`                 | Displays a list of useful make targets with descriptions |
| `build`                | Builds all components with all relevant dependencies in a suitable order |
| `build//<cmpt>`        | Builds a specific component with all relevant dependencies in a suitable order |
| `build//<cmpt>//BUILD` | Builds a specific component, skipping pre-build dependencies. These targets may be useful once the dependencies are already built and you want to rebuild only the component. |
| `check`                | Checks all components' linting, formatting, & other rules |
| `check//<cmpt>`        | Checks a specific component's linting, formatting, & other rules |
| `fix`                  | Updates all linting fixes & formatting for all components. Note that source files will likely be updated as a result. |
| `fix//<cmpt>`          | Updates linting fixes & formatting for a specific component. Note that source files will likely be updated as a result. |
| `test`                 | Tests all components with all relevant dependencies in a suitable order |
| `test//<cmpt>`         | Tests a specific component with all relevant dependencies in a suitable order |
| `test//<cmpt>//TEST`   | Tests a specific component, skipping pre-test dependencies. These targets may be useful once the dependencies are already built and you want to re-test only the component. |
| `prepush`              | Runs all checks & tests for all components |
| `prepush//<cmpt>`      | Runs all checks & tests for a specific component |
| `watch//<cmpt>`        | Runs the default watch task for a specific component |
| `clean`                | Cleans all build/test temporary work files |
| `clean//<cmpt>`        | Cleans all build/test temporary work files for a specific component |
| `run//<cmpt>`          | Runs the default program/server for a specific component with all relevant dependencies built in a suitable order|
| `run//<cmpt>//RUN`     | Runs the default program/server for a specific component, skipping pre-run dependencies |
| `image`                | Builds all container images for relevant components |
| `image//<cmpt>`        | Builds the container for a specific component. Note that not all components have support for building a container image. |
| `release//<cmpt>`      | Builds & pushes the container image for a specific component to the repository. This task is used primarily in CI. |
| `promote//<cmpt>`      | Tags & pushes the current Git revision image as 'stable' for a specific component. This task is used primarily in CI. |
| `ci`                   | Invokes the primary continuous integration task |
| `down`                 | Brings down all supporting services such as databases, queues, etc. Note that this task will destroy any data currently persisted in local databases. |
| `prepare`              | Prepares all supporting services such as databases, queues, etc. for development. Note that this task will destroy any data currently persisted in local databases. |
| `list`                 | Prints a comprehensive list of Make targets, one per line |

#### Common Component Makefile Targets

In the root directory for each sub-component in this project there will be
another Makefile which provides a common set of project-specific tasks. No
matter if the component is Rust code, TypeScript based, or even a Docker image
component, these common targets will usually be present. In this way there
shouldn't be anything dramatically new to learn when exploring another
component in our project.

The inspiration for the common/baseline make targets can be found in Joyent's
[Engineering Guide](https://github.com/Joyent/eng/blob/master/docs/index.md) in
the
[Makefile](https://github.com/Joyent/eng/blob/master/docs/index.md#makefile)
section.

| Target                | Description |
|-----------------------|-------------|
| `help`                | Displays a list of useful make targets with descriptions |
| `build`               | Builds the component. Note that pre-build dependencies are not executed (see Base Makefile Targets). |
| `check-format`        | Checks all code formatting for the component |
| `check-lint`          | Checks all code linting for the component |
| `check`               | Checks all component's linting, formatting, & other rules |
| `fix-format`          | Updates code formatting for the component. Note that source files will likely be updated as a result. |
| `fix-lint`            | Updates code with linting fixes for the component. Note that source files will likely be updated as a result. |
| `fix`                 | Updates all linting fixes & formatting for the component. Note that source files will likely be updated as a result. |
| `test`                | Tests the component. Note that pre-test dependencies are not executed (see Base Makefile Targets). |
| `prepush`             | Runs all checks & tests required before pushing commits related to the component |
| `watch`               | Runs the default watch task for the component |
| `clean`               | Cleans all build/test temporary work files for the component |

#### Runnable Component Makefile Targets

For each "runnable" component (that is, typically most components found under
the `app/` and `bin/` directories), there will be some additional make targets
such as:

| Target                | Description |
|-----------------------|-------------|
| `run`                 | Runs the default program/server for the component. Note that pre-run dependencies are not executed (see Base Makefile Targets). |
| `start`               | Alias for `make run` |

#### Releasable Component Makefile Targets

A subset of components are designed to be built and packaged in Docker images
as a means of producing a deployment artifact. These components are typically
found under the `app/`, `bin/`, and `component/` directories, however not all
components may have support to be built as a container image. Here are some
additional make targets you can expect:

| Target                | Description |
|-----------------------|-------------|
| `release`             | Builds & pushes the container image & tags to the repository. This task is used primarily in CI. |
| `promote`             | Tags & pushes the current Git revision container image as 'stable'. This task is used primarily in CI. |
| `image`               | Builds a container image |
| `publish`             | Builds & pushes the image to the repository |

### Components

#### Rust-Based Components

Generally speaking, most of our Rust crates are members of a single [Cargo
workspace] present at the root of the project in `Cargo.toml`. This means that
all member crates will share the same `Cargo.lock` and output directory (i.e.
the `target/` directory at the root of the project). The member crate locations
are usually found under `bin/` for crates which exist primarily to build a
binary program and under `lib/` for so-called library crates.

[Cargo workspace]: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html

#### TypeScript-Based Components

Components which are composed primarily of TypeScript code are currently using
[npm] for package, dependency, and project management. These components are
usually found under `bin/` for components which are intended to produce a
program/CLI/server, under `lib/` for components which are supporting
libraries/packages, and finally under `app/` for components which may build web
front ends, GUI, or other desktop applications.

[npm]: https://docs.npmjs.com/cli

#### Component Components

There are some perhaps confusingly-named components which can be found under
`component/`. These are typically components which build Docker container
images or other supporting tooling that is ancillary to the project.

## Preparing Your Changes and Running Tests

### Running Checks

You can check a component with:

```bash
make check//bin/veritech
```

Where `bin/veritech` is the path to the component you want to check.

### Running All Checks

You can run all the checks with:

```bash
make check
```

> #### Using Optional Fix Targets
>
> You can (optionally) use the `fix` make targets before running checks.
> Be careful, as the Rust-related fix actions may perform more aggressive fixes than what the check target checks for.
>
> ```bash
> make fix
> ```

### Running Tests

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
make FORCE=true test//lib/sdf
```

### Running the CI tests locally

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

### Running Integration Tests Manually

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

#### Want To See The Live Log Stream?

If you'd like to see a live log stream during test execution, use the `SI_TEST_LOG` variable in conjunction with
the `--nocapture` flag for `cargo test`.

```shell
SI_TEST_LOG=info cargo test -p dal --test integration <your-test> -- --nocapture
```

You can combine this with the `DEBUG` environment variable for `lang-js` for even more information.

```shell
DEBUG=* SI_TEST_LOG=info cargo test -p dal --test integration <your-test> -- --nocapture
```

#### Migrations Running Too Slow? Try Disabling or Choosing Builtin Schema Migrations!

If your integration test does not rely on builtin `Schema(s)` and `SchemaVariant(s)` (e.g. "Docker Image" and
"AWS EC2"), you can disable migrating them by passing in `"none"` or `"false"` (or some variant of them) to the
appropriate environment variable.

```shell
SI_TEST_BUILTIN_SCHEMAS=none cargo test -p dal --test integration <your-test>
```

You can also choose individual builtins to migrate with a comma-separated list.
Feel free to use camel casing for readability, but the individual `Schema` names provided will be converted to all
lowercase letters.
To find which builtin `Schemas` are available for selection, please see the
[file where provided names are evaluated](lib/dal/src/builtins/schema.rs).

```shell
SI_TEST_BUILTIN_SCHEMAS=schemaOne,schemaTwo cargo test -p dal --test integration <your-test>
```

> Note: you may not want to wrap your list in `"` characters, depending on your development environment as they may
> be unintentionally included in the list item(s).

If you want migrations to run as they would by default, remove the environment variable or set it to `"all"` or `"true"`
(or some variant of them).

```shell
SI_TEST_BUILTIN_SCHEMAS=all cargo test -p dal --test integration <your-test>
```

### Debugging Integration Tests with Database Contents

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


-----


**WIP notes on pnpm-based dev workflow**

### setup / installation
- install [pnpm](https://pnpm.io/installation)
  - you'll probably want shell tab completion (`pnpm install-completion`)<br/>
- install js deps for the whole project (`pnpm install` or `pnpm i`)
- `pnpm run docker:deps` - triggers `make prepare` and is aware of apple silicon postgres shenanigans
### running dev stack

> NOTE - `pnpm taskname` or `pnpm run taskname` both work, but tab autocomplete only works with `pnpm run X`

- `pnpm run dev:backend` - runs cargo build (at root) and then boots all 3 backend services in a single terminal
- or alternatively can run individual components. Each script still runs the build at the root level but then boots only that component. Cargo caching means each build after the first is instant.
  - `pnpm run dev:sdf`
  - `pnpm run dev:veritech`
  - `pnpm run dev:pinga`
  - `pnpm run dev:council`
- `pnpm run dev:frontend` - boots frontend for dev - uses vite, autoreload and HMR enabled...
