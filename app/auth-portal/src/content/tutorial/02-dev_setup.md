---
title: Running a development instance of System Initiative
hideWorkspaceLink: true
---

## Run a development instance of System Initiative

First things first - let’s set you up with a System Initiative development environment. You will need to check the
source code out from GitHub, ensure your system has the right toolchains installed, start our ancillary services in
Docker containers, and finally, compile and execute the various System Initiative services.

*Note - If you’re running this in a VM, you’ll want to run through this all as a non-root user with sudo access. On a
personal machine, this will likely be the case already.*

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

### Install development dependencies

To run a development build of System Initiative and complete this tutorial, you will need to have our development
dependencies installed. _This section will install software on your computer and mutate your running environment!_

The following platforms are supported:

| Architecture | Operating System                                 |
|--------------|--------------------------------------------------|
| x86_64       | Arch Linux, Fedora, macOS, Pop!_OS, Ubuntu, WSL2 |
| arm64        | macOS                                            |

If you are running MacOS, you will need [homebrew](https://brew.sh/) installed, along with Rosetta 2 (you can install it
with `softwareupdate --install-rosetta`).

First, you must ensure you have the `rust` and `node` toolchains installed and available in your `PATH`. We recommend
you install rust via [rustup](https://rustup.rs/):

```shell
$ curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

For Node, we recommend managing your toolchain with [volta](https://volta.sh/):

```shell
curl https://get.volta.sh | bash
```

You can now install all the necessary software from within the si directory with:

```shell
$ ./scripts/bootstrap.sh
```

This will ensure you have (at least) the following installed:

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