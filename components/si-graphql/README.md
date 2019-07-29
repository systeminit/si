# System Init GraphQL Server

This the GraphQL Server. It uses apollo-graphl and graphql-modules,
along with Webpack and Typescript.

# Development

## Setup

The default options for the server are the production settings. You probably
don't want that. Create a file called '.env', and populate it thusly:

```
APOLLO_INTROSPECTION=true
APOLLO_PLAYGROUND=true
```

This will provide schema introspection and a default playground.

## Run

You'll need two separate terminal windows to do development.

### Build

```
npm run build
```

This will fire up a webpack build process, and watch the code for hot-reloading
love.

### Serve

```
npm start:env
```

This will start the server and load your `.env` file. It will hot-reload when
webpack recompiles.

### Auth

First, log in to the web UI. Then fire up the console, and type:

```
this.localStorage['authIdToken']
```

That is your auth token. Then head over the the graphql playground on this server,
and in the http headrs section, add

```
{
  "Authorization": "Bearer <YOURSTRING>"
}
```

And viola, you'll be authing your queries.

## Docker Container
Generate a docker container to run this service with `make docker-build` and run the container with `docker-run`. Alternatively you can generate and run the container with `make docker`. 