# Architecture

The diagram (created with [asciiflow](https://asciiflow.com)) below illustrates a _very_ high-level overview of SI's calling stack.
There are other components and paradigms that aren't displayed, but this diagram is purely meant to show the overall flow from "mouse-click" onwards.

```
        ┌───────┐             ┌─────────┐
        │ pinga ├──────┬──────│ rebaser │
        └───────┘      │      └─────────┘
                       │
                       │
┌─────┐   ┌─────┐   ┌──┴──┐
│ web ├───┤ sdf ├───┤ dal │
└─────┘   └─────┘   └──┬──┘
                       │
                       │
                 ┌─────┴────┐
                 │ veritech │
                 └──────────┘
```

## Definitions

- **[web](../app/web/):** the primary frontend web application for SI
- **[sdf](../bin/sdf/):** the backend webserver for communicating with `web`
- **[dal](../lib/dal/):** the library used by `sdf` routes to "make stuff happen" (the keystone of SI)
- **[pinga](../bin/pinga/):** the job queueing service used to execute non-trivial jobs
- **[rebaser](../bin/rebaser/):** where all workspace-level changes are persisted and conflicts are detected based on proposed changes
- **[veritech](../bin/veritech/):** a backend webserver for dispatching functions in secure runtime environments