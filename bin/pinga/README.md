# Pinga

Job Queue Executor integrated with both [Nats](https://nats.io/) (by default) and [Faktory](https://contribsys.com/faktory/) (opt-in)

![pinga](./docs/pinga.png)

## Drip

*[Cachaça](https://en.wikipedia.org/wiki/Cacha%C3%A7a) has thousands of names, a very common one is pinga. It comes from the distillation process, where the distilled alcohol drips into the barrel to be aged.*

![pingando](./docs/pinga.gif)

## Execution

Run pinga:

```
cargo run
```

You can also use the following command in the global workspace:

```
make pinga-run
```

## Faktory

If you decide to swap the transport layer from NATS to Faktory, you will have to have faktory running in the background

Run faktory locally in dev mode with (this doesn't persist the queues):

```
docker run --rm -it -p 127.0.0.1:7419:7419 -p 127.0.0.1:7420:7420 contribsys/faktory:latest
```

