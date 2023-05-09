---
title: Running a development instance of System Initiative
hideWorkspaceLink: true
---

## Run a development instance of System Initiative

First things first - let’s set you up with a System Initiative development environment. You will need to check the
source code out from GitHub, ensure your system has `nix` installed with its environment set up fully, start our
ancillary services in  Docker containers, and finally, compile and execute the various System Initiative services.

*Note - If you’re running this in a VM, you’ll want to run through this all as a non-root user with sudo access. On a
personal machine, this will likely be the case already.*

### Choose a supported platform

To run a development build of System Initiative and complete this tutorial, you will need to have our development
dependencies installed.
Let's start by choosing a compatible platform.

The following platforms are supported:

| Architecture    | Operating System                                                                           |
|-----------------|--------------------------------------------------------------------------------------------|
| x86_64 (amd64)  | macOS, Linux (GNU), [WSL2](https://learn.microsoft.com/en-us/windows/wsl/) (Windows 10/11) |
| aarch64 (arm64) | macOS, Linux (GNU), [WSL2](https://learn.microsoft.com/en-us/windows/wsl/) (Windows 10/11) |

 **Platform Notes:**
 - Using macOS `aarch64 (arm64)` requires on Rosetta 2 (install it with `softwareupdate --install-rosetta`)
 - [NixOS](https://nixos.org/) will not likely work at this time (though, this may be desired in the future)
 - [SELinux](https://en.wikipedia.org/wiki/Security-Enhanced_Linux) will likely need to be set to `permissive` mode or configured to work with `nix`
 - Linux with MUSL instead of GNU *might* work, but it is untested

### Install development dependencies

Once a platform is chosen, we can install the dependencies for using the flake.
_This section will install software on your computer and mutate your running environment!_

- `nix` with flakes enabled
- `docker` from [Docker Desktop](https://www.docker.com/products/docker-desktop/) or [Docker Engine](https://docs.docker.com/engine/)
- (optional, but recommended) [`direnv`](https://direnv.net) version `>= 2.30` hooked into your shell

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

With `nix` and `docker` set up, you will have (at least) the following available:

* automake
* make
* aws-cli
* bash
* butane
* coreutils
* git
* kubeval
* libtool
* protobuf
* skopeo
* jq
* wget
* pnpm

Along with any necessary compiler and development toolchains.

### How to run commands

_For the remainder of the tutorial, all commands need to be run from the `nix` environment._

If `direnv` is installed and [hooked into your shell](https://direnv.net/docs/hook.html), you can `cd` into
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

### Run System Initiative

We require 4 support services, all of which we run out of docker:

* [PostgreSQL](www.postgresql.org)
* [NATS](https://nats.io/)
* [The OpenTelemetry Collector](https://opentelemetry.io/docs/collector/)
* [Jaeger](https://www.jaegertracing.io/)

_Ensure you do not have any of these services currently running - you should stop any existing versions of these
services you may have_.

To start these containers (and delete any data you have in your database as well), please run:

```shell
$ make prepare
```

When the command finishes, you should see:

```shell
Trying PG...ready
```

You are now ready to run the 5 System Initiative specific services: veritech, council, pinga, sdf, and web. We will run
each in a new terminal window.

Start a new terminal, change to the si directory, and run:

```shell
$ make run//bin/veritech
```

You will see output similar to the following:

![Veritech Terminal](/tutorial-img/02-dev_setup/veritech_terminal.png)

When you do, open another terminal, change to the si directory, and run:

```shell
$ make run//bin/council
```

You will see output similar to the following:

![Council Terminal](/tutorial-img/02-dev_setup/council_terminal.png)

Then open another terminal, change to the si directory, and run:

```shell
$ make run//bin/pinga
```

You will see output similar to the following:

![Pinga Terminal](/tutorial-img/02-dev_setup/pinga_terminal.png)

Then open another terminal, change to the si directory, and run:

```shell
$ make run//bin/sdf
```

This process will run database migrations and populate System Initiative with some default assets (we will talk about
assets later in this tutorial!). You will see output similar to the following when it is ready:

![SDF Terminal](/tutorial-img/02-dev_setup/sdf_terminal.png)

You will see the terminal windows for `veritech`, `pinga`, and `council` logging output. Wait for these windows to stop
logging: System Initiative processes the updates applied by `sdf` in the initialization process. Once they have stopped
logging output, you can start our final service - the web interface.

Open another terminal, change to the si directory, and run:

```shell
$ make run//app/web
```

When it finishes, you should see the following in the terminal:

![Web Terminal](/tutorial-img/02-dev_setup/web_terminal.png)

You should now have 5 terminal windows open, running:

* veritech
* council
* pinga
* sdf
* web

The button below should have two green beacons saying "Frontend online" and "Backend online". If you do -
congratulations! You’re running System Initiative. Click the button below to login and open your development workspace:

<!-- must wrap in a div to undo some of the automatic styling -->
<p class="escape"><workspace-link-widget></workspace-link-widget></p>

If you’ve run into trouble - hit us up
on [Discord](https://discord.com/channels/955539345538957342/1080953018788364288), and we’ll get you sorted. In the
future, you can always access your workspace through the <router-link to="/dashboard">dashboard</router-link>.