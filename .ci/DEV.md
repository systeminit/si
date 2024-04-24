# How to run the CI stack locally

First, you need to prepare a local copy of the code you want to test.

```
$ cp ./si ./si-fixme
$ chown -R 2000:2000 ./si-fixme
```

This is to create a working copy of the source that is owned by the same user
as the internal 'ci' user.

## Run the CI platform

```
$ docker-compose -f .ci/docker-compose.test-integration.yml up
```

This will start all the services, but the 'app' service will fail. This is
expected. You can run this from the `si` or `si-fixme` copy, it doesn't matter.

## Run the app service as a container

From your `si-fixme` root directory, you can start the app container:

```
$ docker-compose -f .ci/docker-compose.test-integration.yml run --entrypoint /bin/bash --volume $(pwd):/workdir app
```

That will give you a shell with your current code mounted as /workdir.

To develop:

```
$ nix develop
```

You can now run buck2 as normal and debug the containers in a CI-like environment.
