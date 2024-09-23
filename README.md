# System Initiative

[![Discord Server](https://img.shields.io/badge/discord-gray?style=for-the-badge&logo=discord&logoColor=white)](https://discord.com/invite/system-init)
[![Build dashboard](https://img.shields.io/badge/build%20dashboard-gray?style=for-the-badge&logo=buildkite&logoColor=white)](https://buildkite.com/system-initiative)
[![Main status](https://img.shields.io/buildkite/ecdbcb0ae243a74976f62a95826ec1fce62707e6fe07e4b973?style=for-the-badge&logo=buildkite&label=main)](https://buildkite.com/system-initiative/si-merge-main)

This is a monolithic repository containing the System Initiative software.

## About

System Initiative is a collaborative power tool designed to remove the papercuts from DevOps work.
To learn more, read our ["Second Wave DevOps" blog post](https://www.systeminit.com/blog-second-wave-devops).

## Quickstart

Follow the [Local Development Setup](#local-development-setup) instructions below.
We are working on and investigating more way(s) to try out System Initiative in the future.

## Local Development Setup

Running the System Initiative software locally can be done in a variety of ways, but this abbreviated section will focus on a single method for getting your environment ready to run the stack.
For more information and options on running SI locally, see the [development environment section of the docs](DOCS.md).

### (1) Choose a Supported Platform

Let's start by choosing an officially supported platform.

| Architecture    | Operating System                                                                     |
|-----------------|--------------------------------------------------------------------------------------|
| x86_64 (amd64)  | macOS, Linux, [WSL2](https://learn.microsoft.com/en-us/windows/wsl/) (Windows 10/11) |
| aarch64 (arm64) | macOS, Linux, [WSL2](https://learn.microsoft.com/en-us/windows/wsl/) (Windows 10/11) |

**Platform Notes:**
* On Apple Silicon systems (i.e. macOS aarch64 (arm64)), Rosetta 2 must be installed (install it with `softwareupdate --install-rosetta`)
* [NixOS](https://nixos.org/) requires [`docker`](https://nixos.wiki/wiki/Docker) to be installed and [Flakes](https://nixos.wiki/wiki/Flakes) to be enabled (see the [development environment section of the docs](DOCS.md) for more information)
* Linux with MUSL instead of GNU (e.g. [Alpine Linux](https://www.alpinelinux.org/)) is untested
* Systemd may need to be enabled on WSL2

### (2) Install Dependencies

Install dependencies on your chosen platform.

- **1)** [`nix` with flakes enabled](https://github.com/DeterminateSystems/nix-installer) version `>= 2.18.1` installed
- **2)** `docker` from [Docker Desktop](https://www.docker.com/products/docker-desktop/) or [Docker Engine](https://docs.docker.com/engine/)
- **3a)** [`direnv`](https://direnv.net) version `>= 2.30` installed
- **3b)** [`direnv` hooked into your shell](https://direnv.net/docs/hook.html)

For `nix`, we highly recommend using the [Determinate Nix Installer](https://github.com/DeterminateSystems/nix-installer).

For `docker`, the Docker Desktop version corresponding to your native architecture should be used.
WSL2 users should be able to use either Docker Desktop for WSL2 or Docker Engine inside the WSL2 VM.

For `direnv`, you can install it with [your package manager of choice](https://direnv.net/docs/installation.html).
However, if you're unsure which installation method to use or your package manager does not provide a compatible version, you can use `nix` itself (e.g. `nix profile install nixpkgs#direnv`).

> [!TIP]
> We recommend using [the upstream docs for hooking `direnv` into your shell](https://direnv.net/docs/hook.html), but here is an example on how to do it on a system where `zsh` is the default shell.
> In this example, the following is added to the end of `~/.zshrc`.
>
> ```zsh
> if [ $(command -v direnv) ]; then
>    eval "$(direnv hook zsh)"
> fi
> ```

### (3) Enter the Repository Directory

All commands need to be run from the `nix` environment.
Since `direnv` is installed _and_ hooked into your shell, you can `cd` into the repository and `nix` will bootstrap the environment for you using the flake.

> [!WARNING]
> You may notice a large download of dependencies when entering the repository for the first time.

### (4) (Optional) Configure Docker

Docker Hub authentication is not strictly needed if you only access public docker images, but to avoid being rate-limited when qualifying images, we recommend authenticating with the `docker` CLI.

```bash
docker login
```

### (5) Running the Stack

We use [`buck2`](https://github.com/facebook/buck2) to run the stack, run and build individual services and libraries, perform lints and tests, etc.

_Before continuing, you should stop any locally running services to avoid conflicting ports with the stack.
Some of the services that will run include, but are not limited to the following: PostgreSQL, NATS, Jaeger and OpenTelemetry._

Check if you are ready to run the stack before continuing.

```bash
buck2 run dev:healthcheck
```

You may notice some checks related to resource limits.
On macOS and in WSL2 in particular, we recommend significantly increasing the file descriptor limit for `buck2` to work as intended (e.g. `ulimit -n 10240`).

_Please note: the new file descriptor limit may not persist to future sessions._

Once ready, we can build relevant services and run the entire stack locally.

_Please note: if you have used SI before, the following command will delete all contents of the database.
Reach out to us [on Discord](https://discord.com/invite/system-init) if you have any questions._

```bash
buck2 run dev:up
```

Once Tilt starts, you can check on the status of all services by accessing the UI through the given port on your local host (e.g. [http://localhost:10350/](http://localhost:10350/)).
Every service should eventually have a green checkmark next to them, which ensures that they are in "ready" states.

> [!WARNING]
> _Database migrations may take some time to complete._

If you would like to learn more on what's running, check out the [docs](DOCS.md).
In our documentation, you can also learn more about running the stack locally and a deeper dive into system requirements.

### (6) Troubleshooting in Tilt

If some services failed to start, you can restart them on the Tilt dashboard.

- A backend service fails (e.g. `sdf`): restart them in the following order: `forklift`, `veritech`, `rebaser`, `pinga`, `sdf`
- A frontend service fails (e.g. `web`): restart the service individually
- A dependent service fails (e.g. PostgreSQL): tear down the stack and restart

### (7) Tearing Down the Stack

The following command will stop all running services and containers.
It will also remove the containers and, consequentially, the data held in them.

```bash
buck2 run dev:down
```

Alternatively, if you wish to keep your data for later use, you can stop the containers without removing them.

```bash
buck2 run dev:stop
```

## Where Do I Learn More?

For more information on how to use and develop the System Initiative software, talk to us on
[our Discord](https://discord.com/invite/system-init) and see the [DOCS](DOCS.md).

## How Can I Contribute?

To start, we recommend reading the [Open Source](#open-source) and [Contributing](#contributing) sources below.
They provide information on licensing, contributor rights, and more.

After that, navigate to the [contributing guide](CONTRIBUTING.md) to get started.

## Open Source

This repository contains the System Initiative software, covered under the [Apache License 2.0](LICENSE), except where noted (any System Initiative logos or trademarks are not covered under the Apache License, and should be explicitly noted by a LICENSE file.)

System Initiative is a product produced from this open source software, exclusively by System Initiative, Inc. It is distributed under our commercial terms.

Others are allowed to make their own distribution of the software, but they cannot use any of the System Initiative trademarks, cloud services, etc.

We explicitly grant permission for you to make a build that includes our trademarks while developing the System Initiative software itself. You may not publish or share the build, and you may not use that build to run System Initiative software for any other purpose.

You can [learn more about the System Initiative software and Open Source in our FAQ](https://systeminit.com/open-source).

## Contributing

The System Initiative software is Open Source under the [Apache License 2.0](LICENSE), and is the [copyright of its contributors](NOTICE). If you would like to contribute to the software, you must:

1. Read the [Contributors](CONTRIBUTORS.md) file.
2. Agree to the terms by having a commit in your pull request "signing" the file by adding your name and GitHub handle on a new line at the bottom of the file.
3. Make sure your commits Author metadata matches the name and handle you added to the file.

This ensures that users, distributors, and other contributors can rely on all the software related to System Initiative being contributed under the terms of the [License](LICENSE). No contributions will be accepted without following this process.


### Contributors

<img src="https://markupgo.com/github/systeminit/si/contributors?count=0&circleSize=40&circleRadius=40&center=true" width="100%" /> 
