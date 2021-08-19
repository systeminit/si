# Building Images

> Note this is a work-in-progress document, so feel free to follow up with
> Fletcher if there is detail missing--there almost certainly is!

You'll want to `docker login` when using/building/releasing/publishing these
images as this change uses buildkit/buildx to use Docker Hub as a remote cache.
In other words building on a dev workstation starts with the last updated
buildcache in Docker Hub for that image. If nothing has changed locally, there's
a really good chance Docker will produce a 100% cached image with a new tag
attached. But `docker login` is important. There's also not a great way of
determining whether or not you're logged in, so I didn't add any sanity checks,
sorry!

To build an image for:

- `si-sdf`
- `si-veritech`
- `si-web-app`

simply: `cd components/si-sdf; make image` (this is driven by
`components/si-sdf/bin/build-image.sh` etc.)

To run a component in a container (including any necessary
building/downloading/etc): `cd components/si-sdf; make run-container` (this is
driven by `components/si-sdf/bin/run-container.sh` etc.)

All the dev deps have been updated to the same deal, i.e.
`cd components/otelcol; make run-container`. None of the supporting dev deps
have a build script as we are no longer building custom images, rather using the
stock images upstream with configuration where needed.

The root Makefile has several new targets and a few updated. Importantly:

- `make run-dev-deps`: builds and runs all supporting services in containers,
  same as prior `dev_deps` target
- `make run-containers`: builds and runs all support services and si services in
  containers. You can stop the `web` container and then develop locally on
  si-web-app for example now with the rest of the stack 100% in Docker
  containers
- `make stop-containers`: stops all supporting and si containers
- `make clean-containers`: same as above except the containers are removed--note
  this includes the postgres container

Each runnable component has a set of container make targets:

- `make run-container`: build and run the service in a local Docker container,
  stopped containers are restarted
- `make stop-container`: stops a container that may or may not be running
- `make tail-container`: tails the Docker logs from the container, useful with
  `make run-container tail-container`
- `make clean-container`: stops and removes the named container instance
