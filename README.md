# System Initiative

This is a monolithic repository containing the source for System Initiative (SI)

## Installation

First ensure that Docker is installed on your machine and that the `docker` executable is in your `PATH`.

This section is designed for the following architecture + platform combinations:

* `x86_64  (amd64)`: Arch Linux, Fedora, macOS, Pop!_OS, Ubuntu, WSL2
* `aarch64 (arm64)`: macOS

*aarch64 (arm64) depends on Rosetta 2 to properly execute, install it with `softwareupdate --install-rosetta`*

*If your architecture + platform is not listed above it's probably not supported by the installation script. If that is the case, or you don't want the dependencies to be installed directly in your machine, you can use Nix to encapsulate them. Check the Nix section in the [DEVELOPING](./DEVELOPING.md) document for more information. If doing so, you can skip to the [Configuration](#configuration) section*

To continue the default installation process you must ensure that you have the `rust` toolchain and the `node` toolchain installed and available in your `PATH`. We recommend the following tools to help you manage those toolchains:

* [**rustup**](https://rustup.rs) 🦀: Rust, `cargo`, etc.
* [**volta**](https://volta.sh) ⚡: `node`, `npm`, etc.

Now you can run the bootstrap script, that will install and update all dependencies.

```bash
./scripts/bootstrap.sh
```

For more informations regarding the bootstrap script, or the installation process refer to [DEVELOPING](./DEVELOPING.md). Including troubleshooting and expanding it to support new architectures, or functionalities - generally needed to make more CLI tools available to custom Javascript functions.

*You can also refer to the [docs directory](./docs) for even more details.*

## Configuration

You must authenticate to the AWS console and Docker Hub to ensure System Initiative will work properly.

AWS authentication is required so SI can deploy and manage your infrastructure. Run the following command:

```bash
aws configure
```

Docker Hub authentication is not strictly needed if you only access public docker images, but to avoid being rate-limited when qualifying images, you should probably authenticate with the following command:

```bash
docker login
```

## Running

You must make sure these docker images are running: `postgresql`, `nats`, `opentelemetry`. To do this run the following command:

*If you have any of these services running locally you should stop them to avoid ports conflicting*

*Note: This will delete all of the SI's docker image database's content*

```bash
make prepare
```

Wait for the docker images to be pulled and executed before proceeding.

System Initiative requires 5 processes running to properly execute: `veritech`, `council`, `pinga`, `sdf` and `web`.

*If you can't or don't want to run the processes directly, you can use Nix to encapsulate them. Check [DEVELOPING](./DEVELOPING.md) for more information.*

*Check the section [Architecture](#architecture) for more details about each process.*

Run these in order, each in a new terminal session:

```bash
make run//bin/veritech
```

Wait for `veritech` to finish initializing, otherwise the system might behave weirdly, as custom functions execution will fail.

The following should be be started in order, but the previous doesn't need to be fully initialized for the subsequent to start.

```bash
make run//bin/council
```

```bash
make run//bin/pinga
```

```bash
make run//bin/sdf
```

```bash
make run//app/web
```

After everything is initialized, access it through: http://localhost:8080

*Note: initial compilation times may be long, depending on the machine used*

## Contributing

We highly recommend following the [Convential Commits](https://www.conventionalcommits.org/en/v1.0.0/#specification) format when committing changes.
Our prefixes are derived from the official specification as well as the those found in [commitlint](https://github.com/conventional-changelog/commitlint/tree/master/%40commitlint/config-conventional), based on [Angular's commit conventions](https://github.com/angular/angular/blob/master/CONTRIBUTING.md).
When in doubt, use `feat`, `fix`, or `chore`!

Moreover, please sign your commits using `git commit -s`.
You can amend an existing commit with `git commit -s --amend`, if needed.

## Architecture

The diagram (created with [asciiflow](https://asciiflow.com)) below illustrates a _very_ high-level overview of SI's calling stack.
There are other components and paradigms that aren't displayed, but this diagram is purely meant to show the overall flow from "mouse-click" onwards.

```
                   ┌───────┐   ┌─────────┐
                   │ pinga ├───│ council │
                   └───┬───┘   └─────────┘
                       │
                       │
                       │
┌─────┐   ┌─────┐   ┌──┴──┐   ┌──────────┐
│ web ├───┤ sdf ├───┤ dal ├───┤ postgres │
└─────┘   └─────┘   └──┬──┘   └──────────┘
                       │
      ┌────────────────┘
      │
┌─────┴────┐   ┌──────────────────┐   ┌─────────┐      ┌───────────────────┐
│ veritech ├───┤ deadpool-cyclone ├───┤ cyclone ├ ─ ─> │ execution runtime │
│          │   │                  │   │         │      │ (e.g. lang-js)    │
└──────────┘   └──────────────────┘   └─────────┘      └───────────────────┘
```

### Definitions for Architectural Components

- **[web](./app/web/):** the primary frontend web application for SI
- **[sdf](./bin/sdf/):** the backend webserver for communicating with `web`
- **[dal](./lib/dal/):** the library used by `sdf` routes to "make stuff happen" (the keystone of SI)
- **[pinga](./bin/pinga/):** the job queueing service used by the `dal` to execute non-trivial jobs via `nats`
- **[council](./bin/council/):** the DependentValuesUpdate job's synchronization service, used by `dal` via `nats` to avoid race conditions when updating attribute values
- **[postgres](https://postgresql.org):** the database for storing SI data
- **[nats](https://nats.io):** the messaging system used everywhere in SI, by `pinga`, `council`, `dal` and `sdf` (for multicast websocket events)
- **[veritech](./bin/veritech/):** a backend webserver for dispatching functions in secure runtime environments
- **[deadpool-cyclone](./lib/deadpool-cyclone/):** a library used for managing a pool of `cyclone` instances of varying "types" (i.e. HTTP, UDS)
- **[cyclone](./bin/cyclone/):** the manager for a secure execution runtime environment (e.g. `lang-js`)
- **[lang-js](./bin/lang-js/):** a secure-ish (don't trust it) execution runtime environment for JS functions

It's worth noting that our database has many stored procedures (i.e. database functions) that perform non-trivial logic.
While the [dal](./lib/dal) is the primary "data access layer" for the rest of the SI stack, it does not perform _all_ the heavy lifting.
