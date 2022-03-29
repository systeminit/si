# System Initiative

This is a monolithic repository containing the source for System Initiative (SI).

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

### Notes on `aarch64 (arm64)`

Few SI dependencies rely on using an `x86_64 (amd64)` host.
Fortunately, a compatibility layer, such as [Rosetta 2 on macOS](https://support.apple.com/en-us/HT211861) should suffice during builds.
You can install Rosetta 2 on macOS by executing the following:

```bash
softwareupdate --install-rosetta
```

Despite the above, if any dependency can be made to work on both `aarch64 (arm64)` and `x86_64 (amd64)`, we should do attempt to do so.
Not only is flexibility between architectures useful for local development, but it may also be useful in CI and/or production.

## Quickstart

**Bootstrap:** to get ready to run this repository, you should run the following script:

```bash
./scripts/bootstrap.sh
```

The bootstrapper is (mostly) idempotent, so feel free to run it as many times as you like!
However, it _will_ upgrade existing packages without confirmations, so ensure that you are ready to do so.

**Login:** now, we need to ensure that we are [logged into Docker locally](https://docs.docker.com/engine/reference/commandline/login/) and that the corresponding account can pull images from our [private repositories](https://hub.docker.com/orgs/systeminit/repositories).
Please reach out internally if your account cannot pull images from the private SI repositories.

**Check Services:** SI uses external services in conjunction with its native components.
These external services are deployed via `docker-compose` and are configured to stick to their default settings as closely as possible, including port settings.
Thus, it is worth checking if you are running these services to avoid conflicts when running SI.
Potentially conflicting services include, but are not limited to, the following:

* PostgreSQL DB
* OpenTelemetry
* NATS
* Watchtower

In the case of a port conflict, a good strategy is to temporarily disable the host service until SI is no longer being run.

**Make:** with all dependencies installed and required binaries in `PATH`, we are ready to go!
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

**Teardown**: you can teardown SI and its external services by stopping the active `make` targets above and executing the following in the repository root:

```bash
make down
```

## Preparing Your Changes and Running Tests

Navigate to the `Makefile` in the [ci](./ci) directory to see local development
targets. These targets include code linting, formatting, running CI locally,
etc.

To verify that all lints will pass in CI, execute the following target:

```bash
( cd ci; make ci-lint )
```

> **Optional:** use the "tidy" make targets. Be careful, as the Rust-related
> tidy actions are more aggressive than what the lint target checks for.
>
> ```bash
> ( cd ci; make tidy )
> ```

You can also run individual [DAL](./lib/dal) integration tests before bringing
up the entire SI stack, as neeeded. This can be done in the root of the
repository with two terminal panes.

In one pane, prepare the test environment:

```bash
make prepare; sleep 10; make veritech-run
```

In the other pane, run your test:

```bash
RUST_BACKTRACE=1 cargo test <your-test-name>
```

## Reading and Writing Documentation

Our crates leverage `rustdoc` for seamless integration with `cargo doc`, [IntelliJ Rust](https://www.jetbrains.com/rust/),
[rust-analyzer](https://rust-analyzer.github.io/), and more.

### Reading Rust Documentation

Build the docs for all of our crates and open the docs in your browser at `dal` by executing the following:

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

**Additional resources:**

* [RFC-1574](https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text): more API documentation conventions for `rust-lang`
* ["Making Useful Documentation Comments" from "The Book"](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments): a section of "The Book" covering useful documentation in the context of crate publishing

## Troubleshooting

Having trouble running SI or its tests?
Want to go back to the beginning and wipe the slate clean?
We have a `make` target for that.

```bash
make troubleshoot
```

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
* [Engineering Maxims](https://docs.google.com/document/d/1l-YCyMbXaVAG6VVDucZVJlO7VbJeTAAwt4jB-1usSQA) - some maxims we try to follow

