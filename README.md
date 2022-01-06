# System Initiative

This is a monolithic repository containing the source for System Initiative (SI).

## Supported Developer Environments

* Arch Linux `(x86_64 / amd64)`
* Fedora `(x86_64 / amd64)`
* macOS `(arm64 / aarch64)`
* macOS `(x86_64 / amd64)`
* Ubuntu `(x86_64 / amd64)`
* WSL2 `(x86_64 / amd64)`

> If your preferred environment is not listed, please feel free to add it once the following conditions have been met:
>
> 1. It's been added to the idempotent [bootstrap script](./scripts/bootstrap.sh)
> 2. The aforementioned script has been tested and remains idempotent
> 3. Running the **Quickstart** steps below is successful and the UI is fully functional
>
> _Please note:_ adding your preferred environment will also add you as a maintainer of its functionality throughout this repository.
> If unsure where to start, you can look at a [PR from the past](https://github.com/systeminit/si/pull/589) to help.

We recommend using the latest stable Rust toolchain and latest LTS Node toolchain for your environment.
If unsure, the following tools are recommended to help manage your toolchains:

* [**rustup**](https://rustup.rs) 🦀: Rust, `cargo`, etc.
* [**volta**](https://volta.sh) ⚡: `node`, `npm`, etc.

## Quickstart

**Bootstrap:** to get ready to run this repository, you should run the following script:

```bash
./scripts/bootstrap.sh
```

The bootstrapper is idempotent, so feel free to run it as many times as you like!
However, it _will_ upgrade existing packages without confirmations, so ensure that you are ready to do so.

**Login:** now, we need to ensure that we are [logged into Docker locally](https://docs.docker.com/engine/reference/commandline/login/) and that the corresponding account can pull images from our [private repositories](https://hub.docker.com/orgs/systeminit/repositories).
Please reach out internally if your account cannot pull images from the private SI repositories.

**Make:** with all dependencies installed and required binaries in `PATH`, we are ready to go!
In one terminal pane (e.g. using a terminal multiplexer, such as `tmux`, or tabs/windows), execute the following:

**Services:** docker-compose deploys a postgresql service, an open-telemetry service, a nats service and an watchtower service, using the default ports, so if some of those already are running on the host machine a conflict will happen, for now just disable the host's service and let the docker one do its job

```bash
make prepare
```

This will ensure that our database is running, our NATS server is running, the JS language server is built, and all crates are built.

Now, wait for the `postgres` database container to be running and ready to receive incoming client connection requests.
If it is not ready, `sdf` database migration will fail.

Once the database is ready, you can run `sdf`.

```bash
make sdf-run
```

_Note:_ you can run `sdf` again without running the prepare target if you are only making changes to `sdf` and the libraries it pulls in.

In another terminal pane, execute the following command:

```bash
make app-run
```

This will run the web application, which you can access by navigating to https://localhost:8080.
Now, you have SI running!

## Prepare Your Changes

Navigate to the `Makefile` in the [ci](./ci) directory to see local development targets.
These targets include code linting, formating, running CI locally, etc.
For instance, you can tidy up your Rust code before opening a pull request by executing the following:

```bash
( cd ci; make tidy )
```

You can also run individual tests before bringing up the entire SI stack, as neeeded.
This can be done in the root of the repository:

```bash
make prepare
cargo build
RUST_BACKTRACE=1 cargo test <your-test-name>
``

## Architecture

The diagram below illustrates a _very_ high-level overview of SI's calling stack.
There are many other components, including the JS language server, that aren't displayed, but the "onion-style" diagram is meant to show the overall flow from mouse-click to database entry.

```
┌──────────────────────────┐
│ Web Application          │
│ ┌──────────────────────┐ │
│ │ SDF                  │ │
│ │ ┌──────────────────┐ │ │
│ │ │ DAL              │ │ │
│ │ │ ┌──────────────┐ │ │ │
│ │ │ │ DB ("smart") │ │ │ │
│ │ │ └──────────────┘ │ │ │
│ │ │                  │ │ │
│ │ └──────────────────┘ │ │
│ │                      │ │
│ └──────────────────────┘ │
│                          │
└──────────────────────────┘
```

We claim that the database is "smart" because it includes many functions, currently in `PLPGSQL`, that perform non-trivial logic.

## Contributing

We highly recommend following the [Convential Commits](https://www.conventionalcommits.org/en/v1.0.0/#specification) format when committing changes.
Our prefixes are derived from the official specification as well as the those found in [commitlint](https://github.com/conventional-changelog/commitlint/tree/master/%40commitlint/config-conventional), based on [Angular's commit conventions](https://github.com/angular/angular/blob/master/CONTRIBUTING.md).
When in doubt, use `feat`, `fix`, or `chore`!

Moreover, please sign your commits using `git commit -s`.
You can amend an existing commit with `git commit -s --amend`, if needed.

### Shortcut Integration

If using [Shortcut](https://shortcut.com), you can link stories with commits.
You can do so by including `[sc-XXXX]` in your commit message where `XXXX` represents the story ID.
A recommended workflow is to put it in the body of the commit message rather than the title due to the latter's 50 character limit.

Here is an example commit message using the integration:

```
chore(butt): add I like my butt

- Add I like my butt to butt [sc-1234]
- Enhance unit tests for butt
```

## Engineering Team Links

Welcome to the team! A few handy links:

* [Engineering Team Onboarding](https://docs.google.com/presentation/d/1Ypesl1iZ5KXI9KBxXINYPlo5TexAuln6Dg26yPXEqbM) - the foundation of our team
* [Engineering Process](https://docs.google.com/document/d/1T3pMkTUX5fhzkBpG4NR3x6DrhZ18xXIjnSYl0g6Ld4o) - how we work together 

