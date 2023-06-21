# Developing

This document contains information related to developing and running the SI stack.
However, since it cannot fit everything, you can check out the contents of the [docs directory](./docs) for even more
information.

## Table of Contents

- [Additional Notes on Environment Setup and Running the Stack](#additional-notes-on-environment-setup-and-running-the-stack)
- [Learning About SI Concepts](#learning-about-si-concepts)
- [Repository Structure](#repository-structure)
- [Preparing Your Changes and Running Tests](#preparing-your-changes-and-running-tests)

## Additional Notes on Environment Setup and Running the Stack

Before navigating this section, see the instructions in the root [README](./README.md).

#### Note on Using Direnv

[Direnv](https://direnv.net/) with [nix-direnv](https://github.com/nix-community/nix-direnv) can automatically set up
your shell, which means you don't need to enter a subshell with `nix develop`, or prefix all commands with
`nix develop --command`. There are also plugins to integrate direnv with common editors.

**Editor plugin support:**

- CLion: [Direnv integration](https://plugins.jetbrains.com/plugin/15285-direnv-integration),
  [Better Direnv](https://plugins.jetbrains.com/plugin/19275-better-direnv)
- Emacs: [emacs-direnv](https://github.com/wbolster/emacs-direnv)
- (Neo)Vim: [direnv.vim](https://github.com/direnv/direnv.vim)
- Visual Studio Code: [direnv](https://marketplace.visualstudio.com/items?itemName=mkhl.direnv)

### Troubleshooting Potential Service Conflicts

SI uses external services in conjunction with its native components.
These external services are deployed via [`docker compose`](https://docs.docker.com/compose/) and are configured to stick to their default settings as
closely as possible, including port settings.
Thus, it is worth checking if you are running these services to avoid conflicts when running SI.
Potentially conflicting services include, but are not limited to, the following:

* PostgreSQL DB
* OpenTelemetry
* NATS
* Watchtower

In the case of a port conflict, a good strategy is to temporarily disable the host service until SI is no longer being
run.

### How Will I Know That Each Component Is Ready?

For backend services like `veritech` and `sdf`, there will usually be an `INFO`-level log indicating that the
webserver has bound to a port and is ready to receive messages.
This may be subject to change (e.g. underlying library is upgraded to a new major version and the startup sequence
changes) and will vary from component to component.

### Using CLion Run Configurations Instead of Terminal Panes

This repository contains CLion run configurations for most of these targets, in addition to a `Run SI` compound target
for running all the targets at once. They should be listed on the run menu automatically and are called
`Prepare`, `Run [SDF | Veritech | Pinga | Web]` and `Teardown` (which is related to the next topic).

Using them you should be able to run the whole stack via CLion's Run tool window instead of using multiple shells!

## Learning About SI Concepts

As referenced in [CODE_DOCUMENTATION](docs/CODE_DOCUMENTATION.md), the `rustdoc` static webpages are an entrypoint
into learning about the Rust modules and structs that back many SI concepts.

Let's say you want to learn about what a `Component` is.
You can generate and open the Rust documentation locally via the following command:

```bash
cargo doc --no-deps --document-private-items --open
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

### Components

"Components" correspond to binaries, libraries, releasable bits and services required to run the SI stack.

There are some components found under `component/` that are thin wrappers around external dependencies.
They exist for building `docker` images and/or for supporting tooling that is ancillary to the project.

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

## Preparing Changes and Running Tests

We highly recommend following the [Convential Commits](https://www.conventionalcommits.org/en/v1.0.0/#specification) format when committing changes.
Our prefixes are derived from the official specification as well as the those found in [commitlint](https://github.com/conventional-changelog/commitlint/tree/master/%40commitlint/config-conventional), based on [Angular's commit conventions](https://github.com/angular/angular/blob/master/CONTRIBUTING.md).
When in doubt, use `feat`, `fix`, or `chore`!

Moreover, please sign your commits using `git commit -s`.
You can amend an existing commit with `git commit -s --amend`, if needed.

Please see [the relevant document](docs/PREPARING_CHANGES_AND_RUNNING_TESTS.md) for more information.

## Using `pnpm` for Your Development Workflow

If `Makefiles` and direct `cargo` and `npm` commands aren't your thing, you can use [`pnpm`](https://pnpm.io) for your
development workflow.

First, ensure we have `pnpm` [installed](https://pnpm.io/installation).
You will likely want shell tab completion, which can be installed via the following command:

```shell
pnpm install-completion
```

You will then need to install JS dependencies for the whole project.

```shell
# full option
pnpm install

# shorthand option
pnpm i
```

### Running the Stack with `pnpm`

We'll need to ensure our core services are running first when running the entire stack.

```shell
# This command triggers "make prepare" with awareness to architecture and OS (e.g. will ensure we run PostgreSQL locally
# on Apple Silicon machines)
pnpm run docker:deps
```

You can run tasks similarly to how you would with `npm`.

```shell
# full option (works with tab complete)
pnpm run <taskname>

# shorthand option (does not work with tab complete)
pnpm <taskname>
```

Here are some example tasks that are useful:

- `pnpm run dev:backend`
  - runs `cargo build` at the repository root and then boots all backend services in a single terminal pane
- individual service tasks:
  - each script still runs the build at the root level, but boots only that service
  - caching with `cargo` means each build after the first is instant
  - `pnpm run dev:sdf` (corresponds to [sdf](./bin/sdf))
  - `pnpm run dev:veritech` (corresponds to [veritech](./bin/veritech))
  - `pnpm run dev:pinga` (corresponds to [pinga](./bin/pinga))
  - `pnpm run dev:council` (corresponds to [council](./bin/council))
- `pnpm run dev:frontend`
  - boots frontend for development; uses vite with auto-reload and HMR enabled

### Adding New Crates with `pnpm`

Ensure that `pnpm` will work with our new changes by running the following command:

```shell
node ./scripts/generate-rust-package-json.js
```

This generates new `package.json` and lockfiles for our new and affected crates alike.
