# System Initiative

This is a monolithic repository containing the source for System Initiative (SI).

## Supported Developer Environments

| OS         | OS Type   | amd64 | arm64 (aarch64) |
|------------|-----------|-------|-----------------|
| Arch Linux | linux-gnu | ✅     | 🚫️             |
| Fedora     | linux-gnu | ⚠️    | ⚠️              |
| macOS      | darwin    | ✅     | ✅               |           |

> **Legend:**
> - ✅: validated manually or automatically
> - ⚠️: not yet validated, but likely supported
> - 🚫: not yet validated, decidedly unsupported, and/or yet to be supported

## Quick Start

To get ready to run this repository, you should run the following script:

```bash
./scripts/bootstrap.sh
```

The bootstrapper is idempotent, so feel free to run it as many times as you like!
However, it _will_ upgrade existing packages without confirmations, so ensure that you are ready to do so.

With all dependencies installed and required binaries in `PATH`, we are ready to go!
In one terminal pane (e.g. using a terminal multiplexer, such as `tmux`, or tabs/windows), execute the following:

```bash
make sdf-all
```

This will ensure that our database is running, our NATS server is running, the JS language server is built, all crates are built, and the database has been "warmed up" via our test suite.
Open success, you can execute `make sdf-run` for subsequent runs.

In another terminal pane, execute the following command:

```bash
make app-run
```

This will run the web application, which you can access by navigating to https://localhost:8080.
Now, you have SI running!

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

## Engineering Team Links

Welcome to the team! A few handy links:

* [Engineering Team Onboarding](https://docs.google.com/presentation/d/1Ypesl1iZ5KXI9KBxXINYPlo5TexAuln6Dg26yPXEqbM) - the foundation of our team
* [Engineering Process](https://docs.google.com/document/d/1T3pMkTUX5fhzkBpG4NR3x6DrhZ18xXIjnSYl0g6Ld4o) - how we work together 

