## [buck2](https://github.com/facebook/buck2)

This document contains information on using `buck2` within this repository.
We recommend using the `buck2` binary provided by our [Nix flake](../flake.nix) to ensure compatible versioning.

## Terminology

- A "target" is an instantiation of a rule
- A "rule" is a library-esque function that is can be buildable, runnable and/or testable
- A "buildable" rule (`buck2 build`) only runs when affected sources are changed, and _ignores_ environment variables and passed down command-line arguments
- A "runnable" rule (`buck2 run`) runs upon every invocation, and _accepts_ environment variables and passed down command-line arguments
- A "testable" rule (`buck2 test`) runs upon every invocation and is similar to a runnable rule, but collects test metadata and is intended for sandboxed environments (e.g. CI)

## The Shape of `buck2` Commands

All `buck2` commands follow similar syntax.

```shell
<ENV> buck2 <CMD> <PATH/TO/DIRECTORY/WITH/BUCK>:<TARGET> -- <ARGS>
```

You can use pseudo-relative pathing to access targets.
You cannot use relative parent directories (e.g. `../../path/to/directory/with/BUCK`), but you can
use child relative directories, like in the example below.

```shell
buck2 run lib/dal:test-integration
```

However, you can always use the `//` prefix to start from the root, regardless of your current
working directory in the repository.

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

Both `BUCK` files and the rules it uses are written in Starlark, which is a superset of Python.

There is an upstream [VS Code](https://github.com/facebook/buck2/tree/main/starlark-rust/vscode)
extension that you can build using `npm`.
After building it, you can install the `vsix` file as an extension in VS Code.

One great thing about the extension is that you can use "Go To Definition" in VS Code to follow the path of where a rule
comes from and how it's written.

## Where do the rules come from?

There are two directories where the rules come from:

- **[prelude](../prelude):** the vendored, [upstream standard library](https://github.com/facebook/buck2-prelude) with rules for common use cases and programming languages
  - **Common use case example:** `sh_binary` is provided as a way to run shell scripts
  - **Programming language example:** `rust_library` is a provided as a way to add buildable Rust library targets
  - **Side note:** this must be kept up to date in conjunction with the `buck2` binary
- **[prelude-si](../prelude-si):** our custom library with custom rules and wrappers around existing rules
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

Expanding on the [terminology](#terminology) section, "buildable" targets only use files that are explicitly provided.
If your new file isn't available during builds, you likely need to do one of two things:

- Use the `export_file` rule to ensure the file is available for builds
- Check the `srcs` attribute of a rule, if applicable, to ensure the file is in the sources tree

## What is the difference between `bzl` and `bxl` files?

Sharing the same extension as Bazel files, `bzl` files contain rules written in Starlark for `buck2`.
For complex introspection and interaction with graphs, `bxl` files are Starlark scripts that extend `buck2` functionality.

## How do I run Rust tests with `buck2`?

Check out the [RUNNING_RUST_TESTS](./RUNNING_RUST_TESTS.md) guide!
Essentially, you'll use the following pattern:

```shell
# Pattern for unit tests
<ENV> buck2 run <PATH/TO/RUST/LIBRARY/DIRECTORY/WITH/BUCK>:test-unit -- <ARGS>

# Pattern for integration tests
<ENV> buck2 run <PATH/TO/RUST/LIBRARY/DIRECTORY/WITH/BUCK>:test-integration -- <ARGS>
```

## Where are `buck2` users hanging out?

![Discord Server](https://img.shields.io/badge/discord-5865F2?style=for-the-badge&logo=discord&logoColor=white)

If you are looking to find other `buck2` users and/or ask questions, share ideas and share experiences related to `buck2`, check out the unofficial ["Buck2 Fans" Discord server](https://discord.gg/P5Tbrt735m).