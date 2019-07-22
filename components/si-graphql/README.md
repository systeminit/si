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

