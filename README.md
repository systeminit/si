# System Initiative

This is a monolithic repository containing the source for System Initiative (SI).

## Developing and Running the SI Stack

Please refer to the following documentation for more information:

- **[DEVELOPING](./DEVELOPING.md):** information related to developing and running the SI stack
- **[DOCUMENTATION](./DOCUMENTATION.md):** information related to developer documentation for the contents of this repository

## Contributing

We highly recommend following the [Convential Commits](https://www.conventionalcommits.org/en/v1.0.0/#specification) format when committing changes.
Our prefixes are derived from the official specification as well as the those found in [commitlint](https://github.com/conventional-changelog/commitlint/tree/master/%40commitlint/config-conventional), based on [Angular's commit conventions](https://github.com/angular/angular/blob/master/CONTRIBUTING.md).
When in doubt, use `feat`, `fix`, or `chore`!

Moreover, please sign your commits using `git commit -s`.
You can amend an existing commit with `git commit -s --amend`, if needed.

### Linear Integration

If your pull request addresses a Linear issue in some manner, please refer to the [official guide](https://linear.app/docs/github?tabs=206cad22125a) on linking the two together.

## Engineering Team Links

Welcome to the team! A few handy links:

* [Engineering Team Onboarding](https://docs.google.com/presentation/d/1Ypesl1iZ5KXI9KBxXINYPlo5TexAuln6Dg26yPXEqbM/view) - the foundation of our team
* [The SI Way](https://docs.google.com/document/d/1llbG8MLv2c9SytLnwCrJU27n5yfGsrI1c4Pi6qscVz4/view) - how we work together
* [Engineering Maxims](https://docs.google.com/document/d/1l-YCyMbXaVAG6VVDucZVJlO7VbJeTAAwt4jB-1usSQA/view) - some maxims we try to follow

## Architecture

The diagram (created with [asciiflow](https://asciiflow.com)) below illustrates a _very_ high-level overview of SI's calling stack.
There are other components and paradigms that aren't displayed, but this diagram is purely meant to show the overall flow from "mouse-click" onwards.

```
                   ┌───────┐
                   │ pinga │
                   └───┬───┘
                       │   ┌─────────┐
                       ├───┤ faktory │
                       │   └─────────┘
┌─────┐   ┌─────┐   ┌──┴──┐   ┌──────────┐
│ web ├───┤ sdf ├───┤ dal ├───┤ postgres │
└─────┘   └─────┘   └──┬──┘   └──────────┘
                       │
      ┌────────────────┘
      │
┌─────┴────┐   ┌──────────────────┐   ┌─────────┐      ┌───────────────────┐
│ veritech ├───┤ deadpool-cyclone ├───┤ cyclone ├ ─ ─> │ execution runtime │
│          │   │                  │   │         │      │ (e.g. lang-js)    │
└──────────┘   └──────────────────┘   └─────────┘      └───────────────────┘
```

### Definitions for Architectural Components

- **[web](./app/web/):** the primary frontend web application for SI
- **[sdf](./bin/sdf/):** the backend webserver for communicating with `web`
- **[dal](./lib/dal/):** the library used by `sdf` routes to "make stuff happen" (the keystone of SI)
- **[pinga](./bin/pinga/):** the job queueing service used by the `dal` to execute non-trivial jobs via `faktory`
- **[faktory](https://github.com/contribsys/faktory):** the job queueing mechanism used by `pinga` to execute non-trivial jobs
- **[postgres](https://postgresql.org):** the database for storing SI data
- **[veritech](./bin/veritech/):** a backend webserver for dispatching functions in secure runtime environments
- **[deadpool-cyclone](./lib/deadpool-cyclone/):** a library used for managing a pool of `cyclone` instances of varying "types" (i.e. HTTP, UDS)
- **[cyclone](./bin/cyclone/):** the manager for a secure execution runtime environment (e.g. `lang-js`)
- **[lang-js](./bin/lang-js/):** a secure-ish (don't trust it) execution runtime environment for JS functions

It's worth noting that our database has many stored procedures (i.e. database functions) that perform non-trivial logic.
While the [dal](./lib/dal) is the primary "data access layer" for the rest of the SI stack, it does not perform _all_ the heavy lifting.
