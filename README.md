# System Initiative

This is the source for the System Initiative.

We're currently working on 'BugBear'

![bugbear](https://i.pinimg.com/736x/2b/95/f0/2b95f05d3c62ccd4be854b567a7592e1--fantasy-creatures-mythical-creatures.jpg)

Things are set up as a mono-repo - all you should need to do to start working
is check out this repository. 

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
