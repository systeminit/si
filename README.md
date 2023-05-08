# System Initiative

This is a monolithic repository containing the source for System Initiative (SI).

## Environment Setup

Running SI locally can be done in a variety of ways, but the officially supported method is to use the [Nix Flake](flake.nix)
at the root of the repository.
This section will focus on getting your environment ready to get SI up and running.

### Choose a Supported Platform

Using the flake requires using one of the following platforms:


| Architecture    | Operating System                                                                           |
|-----------------|--------------------------------------------------------------------------------------------|
| x86_64 (amd64)  | macOS, Linux (GNU), [WSL2](https://learn.microsoft.com/en-us/windows/wsl/) (Windows 10/11) |
| aarch64 (arm64) | macOS, Linux (GNU), [WSL2](https://learn.microsoft.com/en-us/windows/wsl/) (Windows 10/11) |

**Platform Notes:**
* Using macOS `aarch64 (arm64)` requires on Rosetta 2 (install it with `softwareupdate --install-rosetta`)
* [NixOS](https://nixos.org/) will not likely work at this time (though, this may be desired in the future)
* [SELinux](https://en.wikipedia.org/wiki/Security-Enhanced_Linux) will likely need to be set to `permissive` mode or configured to work with `nix`
* Linux with MUSL instead of GNU *might* work, but it is untested

### Installation

Once a platform is chosen, we can install the dependencies for using the flake.

1) `nix` with flakes enabled
2) `docker` from [Docker Desktop](https://www.docker.com/products/docker-desktop/) or [Docker Engine](https://docs.docker.com/engine/)
3) (optional, but recommended) [`direnv`](https://direnv.net) version `>= 2.30` hooked into your shell

For `nix`, we highly recommend using the [Zero to Nix](https://zero-to-nix.com/start/install) installer over the
official installer; one reason being that the former will enable flakes by default.

For `docker`, the Docker Desktop version corresponding to your native architecture should be used on macOS.
WSL2 users should be able to use either Docker Desktop for WSL2 or Docker Engine inside the WSL2 VM.
Native Linux or Linux VM users might be able to use `podman` a drop in replacement for `docker`, though this is untested.

For `direnv`, we recommend using it for both ease of running commands and editor integration.
You can install it with [your package manager of choice](https://direnv.net/docs/installation.html), but at least
version `2.30.x` must be used for the flake integration to work properly.
If you're unsure which installation method to use or your package manager does not provide a compatible version, you
can use `nix` itself (e.g. `nix profile install nixpkgs#direnv`).

### Running Commands

All commands need to be run from the `nix` environment.
If `direnv` is installed and [hooked into your shell](https://direnv.net/docs/hook.html), you can `cd` into
the repository and `nix` will boostrap the environment for you using the flake.
Otherwise, you can execute `nix develop` to enter the environment, `nix develop --command <command>` to
execute a command, or use the environment in whatever way your prefer.

### Configuration

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

The following should be started in order, but the previous doesn't need to be fully initialized for the subsequent to start.

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

## Tearing Down the Stack

Stop all `make` targets and run the following command:

```bash
make down
```

The above target will not only stop all running containers, but will remove them as well.

## Additional Information For Environment Setup and Running the Stack

For more information regarding the environment setup process and running the stack, refer to
[DEVELOPING](./DEVELOPING.md).
You can also refer to the [docs directory](./docs) for even more details.

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

## Contributing

System Initiative is Open Source software under the [Apache License 2.0](LICENSE), and is the [copyright of its contributors](NOTICE). If you would like to contribute to System Initiative, you must:

1. Read the [Contributors](CONTRIBUTORS.md) file.
2. Agree to the terms by having a commit in your pull request "signing" the file by adding your name and GitHub handle on a new line at the bottom of the file.
3. Make sure your commits Author metadata matches the name and handle you added to the file.

This ensures that users, distributors, and other contributors can rely on all the software related to System Initiative being contributed under the terms of the [License](LICENSE). No contributions will be accepted without following this process.