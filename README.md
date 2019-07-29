# System Initiative

This is the source for the System Initiative.

We're currently working on 'BugBear'

![bugbear](https://i.pinimg.com/736x/2b/95/f0/2b95f05d3c62ccd4be854b567a7592e1--fantasy-creatures-mythical-creatures.jpg)

Things are set up as a mono-repo - all you should need to do to start working
is check out this repository. 

# Try it out

Make sure you have node installed. Do 'npm install' in each of the components. For
`si-graphql`, run `npm start`. For `si-web-app`, run `npm run serve`. 

Then hit `http://localhost:8080` for the web app. If you want to play around with
GraphQL directly, see the readme there.

# Vagrant Development Environment
Once you have vagrant installed you can fire up a dev environment with `make vagrant-create` and you can destroy the environment with `make vagrant-remove`. Or you can simply use the vagrant commands directly if you know what you're doing.

# Vocabulary

## components

A component is a usable piece of the system. It may be a service, a user
interface, etc.

Components that are user facing should have the 'application' they are a part of
in front of their name. For example, 'si-web' tells us that this is the web
frontend for the 'si' application.

Service and other domain pieces should have '-service' on the end. So 'auth-service',
or 'user-service'.

# Component List

## si-web-app

A [Vue](https://vuejs.org) application that is the Web UI for SI. 

## si-graphql

The GraphQL API.
