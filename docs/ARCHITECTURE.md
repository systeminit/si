# Architecture

The diagram (created with [asciiflow](https://asciiflow.com)) below illustrates a _very_ high-level overview of SI's calling stack.
There are other components and paradigms that aren't displayed, but this diagram is purely meant to show the overall flow from "mouse-click" onwards.

```
                   ┌───────┐   ┌─────────┐
                   │ pinga ├───│ council │
                   └───┬───┘   └─────────┘
                       │
                       │
                       │
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

## Internal Definitions

- **[web](../app/web/):** the primary frontend web application for SI
- **[sdf](../bin/sdf/):** the backend webserver for communicating with `web`
- **[dal](../lib/dal/):** the library used by `sdf` routes to "make stuff happen" (the keystone of SI)
- **[pinga](../bin/pinga/):** the job queueing service used by the `dal` to execute non-trivial jobs via `nats`
- **[council](../bin/council/):** the DependentValuesUpdate job's synchronization service, used by `dal` via `nats` to avoid race conditions when updating attribute values
- **[veritech](../bin/veritech/):** a backend webserver for dispatching functions in secure runtime environments
- **[deadpool-cyclone](../lib/deadpool-cyclone/):** a library used for managing a pool of `cyclone` instances of varying "types" (i.e. HTTP, UDS)
- **[cyclone](../bin/cyclone/):** the manager for a secure execution runtime environment (e.g. `lang-js`)
- **[lang-js](../bin/lang-js/):** a secure-ish (don't trust it) execution runtime environment for JS functions

## External Definitions

- **[postgres](https://postgresql.org):** the database for storing SI data
- **[nats](https://nats.io):** the messaging system used everywhere in SI, by `pinga`, `council`, `dal` and `sdf` (for multicast websocket events)

## Additional Notes

It's worth noting that our database has many stored procedures (i.e. database functions) that perform non-trivial logic.
While the [dal](../lib/dal) is the primary "data access layer" for the rest of the SI stack, it does not perform _all_ the heavy lifting.

