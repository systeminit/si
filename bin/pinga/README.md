# Pinga

Job Queue Executor integrated with [Faktory](https://contribsys.com/faktory/)

![pinga](./docs/pinga.png)

## Drip

*[Cachaça](https://en.wikipedia.org/wiki/Cacha%C3%A7a) has thousands of names, a very common one is pinga. It comes from the distillation process, where the distilled alcohol drips into the barrel to be aged.*

![pingando](./docs/pinga.gif)

## Execution

Run faktory locally in dev mode with (this doesn't persist the queues):

```
docker run --rm -it -p 127.0.0.1:7419:7419 -p 127.0.0.1:7420:7420 contribsys/faktory:latest
```

Run pinga:

```
cargo run
```

You can also use the following command in the global workspace:

```
make run//bin/pinga
```
