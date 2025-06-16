# Documentation

[![Discord Server](https://img.shields.io/badge/discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/invite/system-init)

Welcome to the documentation for the System Initiative software.
This document not only contains information about SI itself, but also about running and developing SI in general.

The sections within range from guides to knowledge-base-esque information.
The goal for the document is to value longer living documentation and notes over precise documentation that reflects a point in time feature of the system.
If you just want to get SI up and running, see the [README](README.md).

Above all, for any and all questions related to either using or developing the software, _never_ hesitate to reach out to us [on our Discord server](https://discord.com/invite/system-init).

# Table of Contents

- [How to Read This Document](#how-to-read-this-document)
- [Development Environment](#development-environment)
- [Troubleshooting](#troubleshooting)
- [Running the Stack Locally](#running-the-stack-locally)
- [Using LocalStack for Secrets and Credentials](#using-localstack-for-secrets-and-credentials)
- [Using buck2](#using-buck2)
- [Reading and Writing Rust-based Code Documentation](#reading-and-writing-rust-based-code-documentation)
- [Learning About SI Concepts](#learning-about-si-concepts)
- [Running Rust Tests](#running-rust-tests)
- [Preparing Your Pull Request](#preparing-your-pull-request)
- [Core Services](#core-services)
- [Repository Structure](#repository-structure)
- [Adding a New Rust Library](#adding-a-new-rust-library)

# How to Read This Document

This file is designed to be rendered on GitHub.
As a result, some sections will not look as intended in all markdown readers.
Here is how to do that:

1. Navigate to [this file on GitHub](https://github.com/systeminit/si/blob/main/DOCS.md).
1. Find the section picker in the file header.
1. Upon clicking it, a sidebar section will pop out with a generated, nested sections that you can click through. GitHub automatically generates these based on the headings for each section and sub-section in this file.

For more information, see the [GitHub guide](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax).

# Development Environment

Developing SI locally can be done in a variety of ways, but the officially supported method is to use the [`nix` flake](flake.nix) at the root of the repository.

## Supported Platforms

Using the flake requires using one of the below platforms.
It is possible that the System Initiative software can be developed on even more platforms, but these platforms have been validated to work with `nix` and the corresponding flake.

### macOS

macOS (Darwin) is officially supported on both x86_64 (amd64) (also known as "Intel") and aarch64 (arm64) (also known as "Apple Silicon") architectures.
We do not specify the minimum version of macOS that must be used, so we recommend looking at the "Dependencies" section for more information.

For aarch64 (arm64) users, [Rosetta 2](https://support.apple.com/en-us/HT211861) must be installed.
You can either install it via directions from the official support page or by running `softwareupdate --install-rosetta`.

On macOS, you will likely hit the file descriptor limit, which requires user intervention.
We have a section further down that describes how to intervene.

### Linux

Linux (GNU) is officially supported on both x86_64 (amd64) and aarch64 (arm64) architectures.
Linux with MUSL instead of GNU is untested.

In general, GNU-based distros will work.
Those include, but are not limited to the following: Ubuntu, Fedora, Debian, Arch Linux, and openSUSE.

#### What about NixOS?

If using NixOS, you need [Docker](https://wiki.nixos.org/wiki/Docker) to be installed and [Flakes](https://wiki.nixos.org/wiki/Flakes) to be enabled.
If not using `direnv`, you can use `nix develop` or [Nix command](https://wiki.nixos.org/wiki/Nix_command).

### Windows

Using native Windows is not supported at this time, but may be desired in the future.
However, [WSL2](https://learn.microsoft.com/en-us/windows/wsl/) on Windows 10 and Windows 11 is officially supported on
both x86_64 (amd64) and aarch64 (arm64) architectures.
In order to work with `nix`, `systemd` may need to be enabled in your WSL2 distro of choice.

On WSL2, you will likely hit the file descriptor limit, which requires user intervention.
We have a section further down that describes how to intervene.

## Tuning the File Descriptor Limit

On some systems, you may need to significantly increase the file descriptor limit for building and running our services with `buck2`.
This is because `buck2` opens many more files than either `cargo` or `pnpm` do.
Not only that, but when using Tilt to build and run concurrent services, even more files are opened than they would be for sequential builds.

Increasing the file descriptor limit is possible via the `ulimit` command.
To see all limits, execute the following command:

```bash
ulimit -a
```

Here is an example of a significant limit increase, where the argument provided after the flag represents the new desired number of file descriptors:

```bash
ulimit -n <file-descriptor-count>
```

To find an acceptable limit, run the health check command.

```bash
buck2 run dev:healthcheck
```

## Tuning the `inotify` kernel setting (Linux Only)

The local development environment sets up a high number of file watches. On Linux, this uses the `inotify` kernel subsystem. This setting
may need to be increased in order to allow all of the `inotify` watches to be configured.

> [!WARNING]
> To tune this value, it will require sudo access.

```bash
sudo sysctl fs.inotify.max_user_watches=<new-max-user-watches-value>
```

To find an acceptable value, run the health check command.

```bash
buck2 run dev:healthcheck
```

To make this setting persistent, consult your distributions documentation.

## Dependencies

For all supported platforms, there are two dependencies that must be installed, `nix` (preferably via the [Determinate Nix Installer](https://github.com/DeterminateSystems/nix-installer)) and `docker`.

### Nix

We use `nix` as our package manager for the repository.
It ensures that our developers are all using the same versions of all packages and libraries for developing SI.

Regardless of how `nix` is installed, it must have the [flakes](https://wiki.nixos.org/wiki/Flakes) feature enabled.
We highly recommend using the [Determinate Nix Installer](https://github.com/DeterminateSystems/nix-installer) over the official installer; one reason being that the former will enable flakes by default.

> [!TIP]
> You can use `direnv` (version >= 2.30) with our [`nix` flake](flake.nix) for both ease of running commands
> and for editor integration.
>
> For more information, see the **Direnv** section.

### Docker

We use `docker` to run our dependent services for the SI stack.
It can either be installed via [Docker Desktop](https://www.docker.com/products/docker-desktop/) or directly via [Docker Engine](https://docs.docker.com/engine/).

For Docker Desktop, the version corresponding to your native architecture should be used (e.g. install the aarch64 (arm64) version on a Apple-Silicon-equipped MacBook Pro).

WSL2 users should be able to use either Docker Desktop for WSL2 or Docker Engine (i.e. installing and using `docker` within the distro and not interacting with the host).

Regardless of platform, you may need to configure credentials in `~/.local/share`.

#### Rancher Desktop

Since [Rancher Desktop](https://rancherdesktop.io/) provides the ability to use [moby](https://github.com/moby/moby), you can use it to run and develop the System Initiative software.
However, it is untested, and you may need to further configuration depending on your platform.

### (Optional) Direnv

[Direnv](https://direnv.net/) (version >= 2.30) with [nix-direnv](https://github.com/nix-community/nix-direnv) can automatically set up  your shell, which means you don't need to enter a subshell with `nix develop`, or prefix all commands with `nix develop --command`.

You can install it with [your package manager of choice](https://direnv.net/docs/installation.html), but if you're unsure which installation method to use or your package manager does not provide a compatible version, you can use `nix` itself (e.g. `nix profile install nixpkgs#direnv`).

We recommend using [the upstream docs for hooking `direnv` into your shell](https://direnv.net/docs/hook.html), but here is an example on how to do it on a system where `zsh` is the default shell.
In this example, the following is added to the end of `~/.zshrc`.

```zsh
if [ $(command -v direnv) ]; then
   eval "$(direnv hook zsh)"
fi
```

There are also plugins to integrate `direnv` with common editors.

#### Editor Plugin Support

- Emacs: [emacs-direnv](https://github.com/wbolster/emacs-direnv)
- IntelliJ-based IDEs: [Direnv integration](https://plugins.jetbrains.com/plugin/15285-direnv-integration), [Better Direnv](https://plugins.jetbrains.com/plugin/19275-better-direnv)
- Neovim and Vim: [direnv.vim](https://github.com/direnv/direnv.vim)
- Visual Studio Code: [direnv](https://marketplace.visualstudio.com/items?itemName=mkhl.direnv)

## How to Run Commands

All commands need to be run from the `nix` environment.
There are two primary options to do so:

1. If `direnv` is installed _and_ hooked into your shell, you can `cd` into the repository and `nix` will bootstrap the environment for you using the flake.
2. Otherwise, you can execute `nix develop` to enter the environment, `nix develop --command <command>` to execute a command, or use the environment in whatever way your prefer.

## How Will I Know That Each Component Is Ready?

For backend services like `veritech` and `sdf`, there will usually be an `INFO`-level log indicating that the webserver has bound to a port and is ready to receive messages.
This may be subject to change (e.g. underlying library is upgraded to a new major version and the startup sequence changes) and will vary from component to component.

# Troubleshooting

This section contains common troubleshooting scenarios when working on the System Initiative software.

## Build Errors Related to Running Services Reliant on `node_modules`

Since we switched to `buck2` for our build infrastructure in mid-2023, you may experience issues when running services reliant on `node_modules` within older cloned instances of the repository.
To solve these build errors, run the following in the root of your repository:

> [!WARNING]
> *This command deletes files.*
> Ensure your current working directory is the root of the repository and understand what the command does before executing.
> Please reach out to us [on our Discord server](https://discord.com/invite/system-init) if you have any questions.

```bash
find app bin lib third-party/node -type d -name node_modules -exec rm -rf {} \;; rm -rf node_modules
```

## NATS Jetstream Not Enabled

If you see an error related to [NATS Jetstream](https://docs.nats.io/nats-concepts/jetstream) not being enabled when running the stack or tests, your local [`systeminit/nats`](https://hub.docker.com/repository/docker/systeminit/nats/) image is likely out of date.
To get the most up-to-date images, including the aforementioned image, run the following command:

```bash
buck2 run //dev:pull
```

## Potential Service Conflicts

SI uses external services in conjunction with its native components.
These external services are deployed via [`docker compose`](https://docs.docker.com/compose/) and are configured to stick to their default settings as closely as possible, including port settings.
Thus, it is worth checking if you are running these services to avoid conflicts when running SI.
Potentially conflicting services include, but are not limited to, the following:

* Jaeger
* NATS and NATS Jetstream
* OpenTelemetry
* PostgreSQL

In the case of a port conflict, a good strategy is to temporarily disable the host service until SI is no longer being run.

## Seeing Errors Related to Procedural Macros

In your editor, you may find that you'll see errors like `"YourEnum does not implement Display"` if you are using [`Display` from the `strum` crate](https://docs.rs/strum/latest/strum/derive.Display.html).
This is because your editor may not have "proc macros" ("procedural macros") enabled.
Check out your editor or relevant plugin docs for more information.

# Running the Stack Locally

This section provides additional information on running the System Initiative software stack locally.
Readers should first refer to the [README](README.md) and the development environment section higher up before reading this document.

## Advanced Options

While the [README](README.md) covers using `buck2 run dev:up`, there are two other ways to run the full stack locally:

- `buck2 run dev:up-standard`: run with `rustc` default build optimizations
- `buck2 run dev:up-debug`: run with `rustc` debug build optimizations for all services except for the `rebaser`
- `buck2 run dev:up-debug-all`: run with `rustc` debug build optimizations for all services

By default, the stack will run with `rustc` release build optimizations, which is what users and testers of the System Initiative software will want to use.
It runs the software in its intended state.
However, if you are a contributor seeking build times suitable for rapid iteration, you may want to use one of the aforementioned options.

> [!WARNING]
> _Contributors should test your changes with integration tests and with release optimizations when running the full stack._
> _The aforementioned options are solely recommended for rapid iteration._

## What if I want to access my local instance remotely?

You run the full stack the same way, but with additional environment variables.

```shell
TILT_HOST=0.0.0.0 DEV_HOST=0.0.0.0 buck2 run dev:up
```

> [!CAUTION]
> _The user is responsible for maintaining secure access (local, remote, etc.) to their local instance._

> [!NOTE]
> Unless you are using "localhost", your remote workspace must be accessible using SSL.
> For example, if you are using [Tailscale](https://tailscale.com/) and running SI on a remote instance without SSL, you may want to port foward over SSH and use a local development workspace instead.

# Using LocalStack for Secrets and Credentials

This section contains information related to using [LocalStack](https://github.com/localstack/localstack) when working on the System Initiative software.

## How to Use with the "AWS Credential" Builtin `SchemaVariant`

You can use the "AWS Credential" builtin `SchemaVariant` with LocalStack when running the System Initiative software with the following command:

```shell
buck2 run //dev:up
```

To use LocalStack with "AWS Credential", do the following:

1. Create a `Component` using the `SchemaVariant`.
1. Create a `Secret` and use it in the property editor.
1. Populate `http://localhost:4566` (or `http://0.0.0.0:4566`, depending on your system) in the "Endpoint" field for the `Secret`.

Now, you can use LocalStack in your development setup.

# Using buck2

This section contains information on using `buck2` within this repository.
We recommend using the `buck2` binary provided by our [`nix` flake](flake.nix) to ensure compatible versioning.

For information on what `buck2` is, please refer to the [upstream repository](https://github.com/facebook/buck2).

## Terminology

- A "target" is an instantiation of a rule
- A "rule" is a library-esque function that can be buildable, runnable and/or testable
- A "buildable" rule (`buck2 build`) only runs when affected sources are changed, and _ignores_ environment variables and passed down command-line arguments
- A "runnable" rule (`buck2 run`) runs upon every invocation, and _accepts_ environment variables and passed down command-line arguments
- A "testable" rule (`buck2 test`) runs upon every invocation and is similar to a runnable rule, but collects test metadata and is intended for sandboxed environments (e.g. CI)

## The Shape of `buck2` Commands

All `buck2` commands follow similar syntax.

```shell
<ENV> buck2 <CMD> <PATH/TO/DIRECTORY/WITH/BUCK>:<TARGET> -- <ARGS>
```

You can use pseudo-relative pathing to access targets.
You cannot use relative parent directories (e.g. `../../path/to/directory/with/BUCK`), but you can use child relative directories, like in the example below.

```shell
buck2 run lib/dal:test-integration
```

However, you can always use the `//` prefix to start from the root, regardless of your current working directory in the repository.

```bash
# Let's change our current working directory to somewhere in the repository.
cd bin/lang-js

# Now, let's build using a BUCK file somewhere else in the repository.
buck2 build //lib/dal
```

You may have noticed in the example above, we could build `lib/dal` without writing `lib/dal:dal`.
If the target shares the same name as the directory, you do not have to write the name.

```bash
# This...
buck2 build lib/dal

# is the same as this...
buck2 build lib/dal:dal

# and this...
buck2 build //lib/dal

# is the same as this.
buck2 build //lib/dal:dal
```

## I don't have syntax highlighting when viewing `BUCK` or prelude files. Help!

Both `BUCK` files and the rules it uses are written in [Starlark](https://github.com/bazelbuild/starlark), which is a superset of Python.

There is an [upstream VS Code extension](https://github.com/facebook/buck2/tree/main/starlark-rust/vscode) that you can build locally using `npm`.
After building it, you can install the `vsix` file as an extension in VS Code.

One great thing about the extension is that you can use "Go To Definition" in VS Code to follow the path of where a rule comes from and how it's written.

## Where do the rules come from?

There are two directories where the rules come from:

- **[prelude](prelude):** the vendored, [upstream standard library](https://github.com/facebook/buck2-prelude) with rules for common use cases and programming languages
  - **Common use case example:** `sh_binary` is provided as a way to run shell scripts
  - **Programming language example:** `rust_library` is a provided as a way to add buildable Rust library targets
  - **Side note:** this must be kept up to date in conjunction with the `buck2` binary
- **[prelude-si](prelude-si):** our custom library with custom rules and wrappers around existing rules
  - **Example:** `rust_library` provides more than the standard `rust_library` rule and includes additional targets

## How do I view all targets available for a given directory?

Run the following command, but do not forget the `:` at the end.

```shell
buck2 targets <PATH/TO/DIRECTORY/WITH/BUCK>:
```

Here is an example:

```shell
buck2 targets //lib/dal:
```

## Why isn't this new file I added available during builds?

Expanding on the terminology section, "buildable" targets only use files that are explicitly provided.
If your new file isn't available during builds, you likely need to do one of two things:

- Use the `export_file` rule to ensure the file is available for builds
- Check the `srcs` attribute of a rule, if applicable, to ensure the file is in the sources tree

## What is the difference between `bzl` and `bxl` files?

Sharing the same extension as Bazel files, `bzl` files contain rules written in Starlark for `buck2`.
For complex introspection and interaction with graphs, `bxl` files are Starlark scripts that extend `buck2` functionality.

## How do I run Rust tests with `buck2`?

Check out the section on running Rust tests farther down in this file.
Short version: you'll use the following pattern:

```shell
# Pattern for unit tests
<ENV> buck2 run <PATH/TO/RUST/LIBRARY/DIRECTORY/WITH/BUCK>:test-unit -- <ARGS>

# Pattern for integration tests
<ENV> buck2 run <PATH/TO/RUST/LIBRARY/DIRECTORY/WITH/BUCK>:test-integration -- <ARGS>
```

## How do I build all Rust targets with `buck2`?

With `xargs` installed, run the following command:

```shell
buck2 uquery 'kind("rust_(binary|library|test)", set("//bin/..." "//lib/..."))' | xargs buck2 build
```

This commands queries for all `rust_binary`, `rust_library` and `rust_test` targets found in `bin/**` `lib/**` directories with `BUCK` files.
Then, it pipes that output into `buck2 build`.

## How do I build all runnable services written in Rust with `buck2`?

If you intend to run these services with optimial performance, you need to build with release mode.
Here is an example building `sdf`, `pinga`, `veritech`, `rebaser`, `forklift` and `edda` from the repository root:

```shell
buck2 build @//mode/release bin/sdf bin/pinga bin/veritech bin/rebaser bin/forklift bin/edda
```

## What are modes used in `buck2` builds?

Specifying modes (i.e. `@//mode/<mode>`) from the [mode](mode) directory changes the build optimizations for buildable targets.
The build cache is not shared between modes and changing modes does not invalidate the cache for other modes.
In other words, you build the same buildable target with two different modes and retain both caches.

## Where are `buck2` users hanging out?

If you are looking to find other `buck2` users and/or ask questions, share ideas and share experiences related to `buck2`, check out the unofficial ["Buck2 Fans" Discord server](https://discord.gg/P5Tbrt735m).

# Reading and Writing Rust-based Code Documentation

This section contains all information related to developer documentation for this repository's source code.

## Quickstart

Let's say you want to learn what a `Component` or an `AttributeValue` is.
Where do you look?
You can generate and open the docs in your browser to find out!

```bash
buck2 run //lib/dal:doc -- --document-private-items --open
```

Our Rust crates contain module and documentation comments that can be generated into static webpages by `rustdoc`.
When in doubt, see if `doc` target for a Rust-based library has what you are looking for.

## How Do We Generate Rust Documentation?

As previously mentioned, for our Rust crates, we leverage `rustdoc` for seamless integration with [IntelliJ Rust](https://www.jetbrains.com/rust/), [rust-analyzer](https://rust-analyzer.github.io/), and more.

## Writing Rust Documentation

We try to follow the official ["How to write documentation"](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html) guide from `rustdoc` as closely as possible.
Older areas of the codebase may not follow the guide and conventions derived from it.
We encourage updating older documentation as whilst navigating through SI crates.

### Additional Resources

* [RFC-1574](https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text): more API documentation conventions for `rust-lang`
* ["Making Useful Documentation Comments" from "The Book"](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments): a section of "The Book" covering useful documentation in the context of crate publishing

# Learning About SI Concepts

As referenced in our code documentation section, the `rustdoc` static webpages are an entrypoint into learning about the Rust modules and structs that back many SI concepts.

Let's say you want to learn about what a `Component` is.
You may want to open the docs for the [dal](lib/dal) as it is the center of many SI concepts.

You can generate and open the Rust documentation locally via the following command:

```bash
buck2 run lib/dal:doc -- --document-private-items --open
```

# Running Rust Tests

This section contains information related to running Rust tests.

## Setup

Before running Rust based tests, we should ensure that dependent services are running.

```bash
buck2 run dev:platform
```

> [!TIP]
> _Not all tests require dependent services (e.g. unit tests for a small library), but if you are unsure, we recommend running dependent services in advance._

## Running All Tests

For a given crate, you can use `buck2 test`.

```bash
buck2 test <crate>:test
```

> [!TIP]
> The `<crate>` has the structure `//{directory}/{package}` (e.g. `//lib/sdf-server`).

To see all options, use `buck2 targets` with a `:` character at the end.

```bash
buck2 targets <crate>:
```

To list all targets run:

```
buck2 targets //bin/...
buck2 targets //lib/...
```

> [!TIP]
> *Ellipsis also work for other directories.*
> *This works for the root as well, but the list at root will contain a lot of noise.*
>
> ```bash
> buck2 targets ...
> ```

## Running Individual Tests

You can also run individual tests, as needed.

Let's start with running all tests for a given package.

```bash
cd lib/dal
buck2 test :test
```

What if you want to run an individual test for `lib/dal`?
Instead of using `buck2 test`, we will use `buck2 run`.

Here is an example with an individual [dal](lib/dal) integration test:

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

# Preparing Your Pull Request

This section contains information related to preparing changes for a pull request.

## Commit Message Format

We do not require a particular commit message format of any kind, but we do require that individual commits be descriptive, relative to size and impact.
For example, if a descriptive title covers what the commit does in practice, then an additional description below the title is not required.
However, if the commit has an out-sized impact relative to other commits, its description will need to reflect that.

Reviewers may ask you to amend your commits if they are not descriptive enough.
Since the descriptiveness of a commit is subjective, please feel free to talk to us [on our Discord server](https://discord.com/invite/system-init) if you have any questions.

### Optional Commit Template

If you would like an optional commit template, see the following:

```text
<present-tense-verb-with-capitalized-first-letter> <everything-else-without-punctuation-at-the-end>

<sentences-in-paragraph-format-or-bullet-points>
```

Here is an example with a paragraph in the description:

```text
Reduce idle memory utilization in starfield-server

With this change, starfield-server no longer waits for acknowledgement
from the BGS API. As soon as the request is successful, the green
thread is dropped, which frees up memory since the task is usually idle
for ~10 seconds or more.
```

Here is another example, but with bullet points in the description:

```text
Replace fallout queue with TES queue

- Replace fallout queue with TES queue for its durability benefits
- Refactor the core test harness to use TES queue
- Build and publish new TES queue Docker images on commits to "main"
```

Finally, here is an example with a more complex description:

```text
Use multi-threaded work queue operations in starfield-server

Iterating through work queue items has historically been sequential in
starfield-server. With this change, rayon is leveraged to boost overall
performance within green threads.

starfield-server changes:
- Replace sequential work queue with rayon parallel iterator

Test harness changes:
- Refactor the core test harness to create an in-line work queue
```

## Guide: Rebasing Your Changes

This is an opinionated guide for rebasing your local branch with the latest changes from `main`.
It does not necessarily reflect present-day best practices and is designed for those who would like to perform the
aforementioned action(s) without spending too much time thinking about them.

1. Ensure you have [VS Code](https://code.visualstudio.com/) installed.
2. Ensure your local tree is clean and everything is pushed up to the corresponding remote branch.
    1. This will make it easier if we want to see the diff on GitHub later.
3. Open VS Code and create a new terminal within the application.
    1. We will execute this guide's commands in this terminal in order to get `CMD + LEFT CLICK` functionality for files with conflicts.
4. Run `git pull --rebase origin main` to start the process.
    1. If there is at least “conflict area” for that one commit that git cannot figure out, it’ll drop you into interactive rebase mode.
    2. It will keep you in interactive rebase until you have finishing “continuing” through all the commits.
5. Run `git status` to see what is wrong and where there are conflicts.
6. Open all files with conflicts by clicking `CMD + LEFT CLICK` on each one.
7. In each “conflict area” in a given file, you’ll have options (font size is small) at the top to help resolve the conflict(s).
    1. Affected files are marked with a red exclamation point in the VS Code file picker.
    2. In those options, “Current” refers to `HEAD`, which is `main` in our case.
    3. In those same options, “Incoming” refers to changes on our branch.
    4. You can the options or manually intervene to make changes. Sometimes, you may want to accept everything on HEAD or your local branch and just triage manually. Sometimes, you’ll want to not accept anything and manually triage the whole thing. Sometimes you’ll want to do both. It depends!
    5. Finally, it can be useful to have your branch diff open on GitHub to see what you changed before the rebase: `https://github.com/systeminit/si/compare/main...<your-branch>`.
8. Once all conflict areas for “unmerged paths” (files with conflicts) have been resolved, run `git add` with either the entire current working directory and below (`.`) or specific files/directories (e.g. `lib/dal/src lib/sdf-server/src/`) as the next argument(s).
9. Now run `git status` again. The output should indicate that conflicts have been resolved and that we can continue rebasing.
10. If everything looks good in the output, run `git rebase --continue`. You will have an opportunity to amend your commit message here, if desired.
    1. You will not have to necessarily the “human fix this conflict area” process for every commit.
    2. It will only happen for commits with conflict areas.
11. Once the interactive rebase ends (or never even started if there were no conflicts), you should be good to push! Now, run `git push`.
    1. You will likely have to add the `-f/--force` flag since we are overwriting history (technically?) on the remote.
    2. Be careful when using the force flag! Try to push without using the force flag first if you are unsure.
12. You are done! Congratulations!

## Guide: Squashing Your Changes

This is an opinionated guide for squashing the commits on your local branch and pushing them to
your corresponding remote branch.
It does not necessarily reflect present-day best practices and is designed for those who would like to perform the
aforementioned action(s) without spending too much time thinking about them.

1. Ensure your local tree is clean and everything is pushed up to the corresponding remote branch.
    1. This will make it easier if we want to see the diff on GitHub later.
2. Count the number of commits that you'd like to squash.
    1. Navigating to your branch diff on GitHub can be helpful here: `https://github.com/systeminit/si/compare/main...<your-branch-name>`
3. Run `git reset --soft HEAD~N` where `N` is the number of commits (example: `git reset --soft HEAD~2` where you'd like to squash two commits into one).
4. Run `git status` to see all staged changes from the commits that were soft reset.
5. Now, commit your changes (e.g. `git commit -s`).
6. Finally, run `git push`.
    1. You will likely have to add the `-f/--force` flag since we are overwriting history (technically?) on the remote.
    2. Be careful when using the force flag! Try to push without using the force flag first if you are unsure.
7. You are done! Congratulations!

# Core Services

This section contains the paths and brief definitions of the services that run in the System Initiative software stack.

- **[edda](bin/edda/):** builds materialized views of graph-based objects for the frontend
- **[forklift](bin/forklift/):** the service that forklifts data from SI to a data warehouse (or perform an "ack and no-op")
- **[pinga](bin/pinga/):** the job queueing and execution service used to execute non-trivial jobs
- **[rebaser](bin/rebaser/):** where all workspace-level changes are persisted and conflicts are detected based on proposed changes
- **[sdf](bin/sdf/):** the backend webserver for communicating with `web` that contains the majority of the "business logic"
- **[veritech](bin/veritech/):** a backend webserver for dispatching functions in secure runtime environments
- **[web](app/web/):** the primary frontend web application for using the System Initiative software

# Repository Structure

While there are other directories in the project, these are primarily where most of the interesting source code lives and how they are generally organized:

| Directory    | Description                                                    |
|--------------|----------------------------------------------------------------|
| `app/`       | Web front ends, GUIs, or other desktop applications            |
| `bin/`       | Backend programs, CLIs, servers, etc.                          |
| `component/` | Docker container images and other ancillary tooling            |
| `lib/`       | Supporting libraries and packages for services or applications |

# Adding a New Rust Library

To add a new Rust library, there are a few steps:

1. Create `lib/MY-LIBRARY` and put a `Cargo.toml` there like this:

```toml
[package]
name = "MY-LIBRARY"
edition = "2024"
version.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
rust-version.workspace = true
publish.workspace = true
```

2. Put `src/lib.rs` with your code.
3. Add your library to the top-level `Cargo.toml` inside `[workspace] members = ...`.
4. Run `cargo check --all-targets --all-features` to get your library added to the top level `Cargo.lock`.

> [!NOTE]
> If your library adds, removes or modifies a third party crate, you may need to sync `buck2` deps.
> See the [support/buck2](support/buck2) directory for more information.
