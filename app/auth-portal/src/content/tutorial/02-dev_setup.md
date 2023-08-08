---
title: Running a development instance of System Initiative
hideWorkspaceLink: true
---

## Run a development instance of System Initiative

First things first - let’s set you up with a System Initiative development environment. You will need to check the
source code out from GitHub, ensure your system has `nix` installed with its environment set up fully (*we highly
recommend using the [Determinate Nix Installer](https://github.com/DeterminateSystems/nix-installer) over the  official installer*), start
our ancillary services in Docker containers, and finally, compile and execute the various System Initiative services.

*Note - If you’re running this in a VM, you’ll want to run through this all as a non-root user with sudo access. On a
personal machine, this will likely be the case already.*

### Choose a supported platform

To run a development build of System Initiative and complete this tutorial, you will need to have our development
dependencies installed.
The following  platforms are supported: macOS, Linux (GNU), [WSL2](https://learn.microsoft.com/en-us/windows/wsl/)
(Windows 10/11) on either `x86_64 (amd64)` or `aarch64 (arm64)`.

**macOS Notes:**
- On Apple Silicon systems (i.e. macOS `aarch64 (arm64)`), Rosetta 2 must be installed (install it with `softwareupdate --install-rosetta`)
- On either architecture, you may need to run `xcode-select --install` before proceeding

### Install development dependencies

Once a platform is chosen, we can install the dependencies required for using the Nix flake.
_This section will install software on your computer and mutate your running environment!_

- `nix` with flakes enabled (enabled by defualt when using the recommended
  [Determinate Nix Installer](https://github.com/DeterminateSystems/nix-installer))
- `docker` from [Docker Desktop](https://www.docker.com/products/docker-desktop/) or [Docker Engine](https://docs.docker.com/engine/)
  corresponding to your native architecture (WSL2 users can use either Docker Desktop for WSL2 or Docker Engine inside
  WSL2)
- [`direnv`](https://direnv.net) version `>= 2.30` hooked into your shell

### How to run commands

_For the remainder of the tutorial, all commands need to be run from the `nix` environment._

With `direnv` installed and [hooked into your shell](https://direnv.net/docs/hook.html), you can `cd` into
the repository and `nix` will boostrap the environment for you using the flake.
Otherwise, you can execute `nix develop` to enter the environment, `nix develop --command <command>` to
execute a command, or use the environment in whatever way your prefer.

### Configure dependencies

System Initiative will interact with AWS through the AWS CLI and inspect containers via Docker Hub. To configure your
AWS credentials:

```shell
$ aws configure
```

Docker Hub authentication is not strictly needed, as we will only be pulling public docker images - but to avoid being
rate limited, you should authenticate:

```shell
$ docker login
```

### Check out the source code

The [System Initiative software is hosted on GitHub](https://github.com/systeminit/si). Open a terminal window, decide
what directory you want to put the source code in, and check the source code out with:

```shell
$ git clone https://github.com/systeminit/si.git
```

Once git has finished checking out your code, the rest of these steps take place within the directory you just checked out.

```shell
$ cd si
```

We will refer to this as the `si directory` for the rest of the tutorial.

### Dev Environment Healthcheck

If you want to ensure that you are setup and ready to run System Initiative, please run the command:

```shell
$ buck2 run dev:healthcheck
```

This will give you an output with a list of remediations to take before running System Initiative.

### Run System Initiative

Now we have the source code, the dependencies and checked that the system is ready to run it, you can run a development
environment of System Initiative.

Please note that the development environment runs some supporting "platform" services, all of which we run out of docker:

* [PostgreSQL](www.postgresql.org)
* [NATS](https://nats.io/)
* [The OpenTelemetry Collector](https://opentelemetry.io/docs/collector/)
* [Jaeger](https://www.jaegertracing.io/)

As such, ensure you do not have any of these services currently running - you should stop any existing versions of these
services you may have.

To run a development environment, please run the following:

```shell
buck2 run dev:up
```

This will use a [tilt file](https://tilt.dev/) to bring up the correct services in the correct order. You can follow the
prompt in the terminal to open the tilt console. The tile console will show what services are running. When tilt tells us
that 10/10 services are running, the System Initiative is fully running.

The button below should have two green beacons saying "Frontend online" and "Backend online". If you do -
congratulations! You’re running System Initiative. Click the button below to login and open your development workspace:

<!-- must wrap in a div to undo some of the automatic styling -->
<p class="escape"><workspace-link-widget></workspace-link-widget></p>

If you’ve run into trouble - hit us up
on [Discord](https://discord.com/channels/955539345538957342/1080953018788364288), and we’ll get you sorted. In the
future, you can always access your workspace through the <router-link to="/dashboard">dashboard</router-link>.

### Stopping the Development Environment

To stop the development environment, please use `ctrl+c` in the terminal running the `buck2` command. Note that this
will leave the platform services running (such as PostgreSQL, NATS, the OpenTelemetry collector, etc.). To stop those
platform services, you can run the command:

```shell
$ buck2 run dev:down
```
