# System Initiative

This is the source for the System Initiative.

## Quick Start

### Linux 

This repository is known to only work on Arch Linux or Ubuntu. If you're 
trying to run it on something else... sorry, it's not supported.

If you don't have a Linux VM handy, you can use the scripts in the ./scripts/ folder to get yourself a docker container. Install docker, then run the script, and you'll have the repo mounted inside a container. Follow the instructions for bootstrapping, and you're gtg.

### Bootstrapping

To get ready to run this repository, you should run:
```
./components/build/bootstrap.sh
```

This will detect either Arch or Ubuntu, and install the pre-requisites
needed to build a component.

Next, you need to install the dependencies.

```
make build_deps
```

Next, you should run:

```
make build
```

This will ensure that all the pre-requisites for each component are 
installed, and compile each component. If this is successful, 
congratulations, you're all almost done!

### Tests

Last stop - get the test suite working. 

*Make sure you have docker installed and running as a service.*

Build a local couchbase container for your development work:

```
cd ./components/couchbase
./build.sh
```

Then run it!

```
./components/couchbase/run.sh
```

You can then navigate to the Couchbase Web UI:

```
http://localhost:8091
```

Log in as user `si`, password `bugbear`. You should see two buckets - `si` and `si_integration`.

Also start Jaeger, to get distributed tracing:

```
./components/jaeger/run.sh
```

Navigate to the Jaeger web ui:

```
http://localhost:16686
```

Then, you can do:

```
env RUST_LOG=debug make test
```

Assuming the tets pass - congratulations!

### Starting the services

1. Make sure your `db` container is started (docker ps)
2. Start the account servcice: cd ./components/si-account && make start
3. Start the graphql api: cd ./components/si-graphql-api && make start
4. Start the web ui: cd ./components/si-web-ui && make start

### Create an account

1. Hit sign up, create an account, and log in.

You'll get a bug - no matter what you do, you won't be able to log in.
To fix it, go to Couchbase, log in, and go to the "Query" screen. You
need to type:

```
create primary index on `si`
```

And hit execute. Then you can log in, and you'll be gtg.

## Regular use

```
make tmux
```

Will start all the services in a tmux session called `si`. You can also
specifically ask for windows or panes, with:

```
make tmux//windows
```

Or 

```
make tmux//panes
```

If a session does not exist, one will be created for you. If you are inside of
tmux already, it will automatically detect that, and create the panes/windows
for you inside the session you are in.

```
make watch
```

Will start all the services in a single shell, without tmux.
