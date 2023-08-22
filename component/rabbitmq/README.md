# RabbitMQ

This directory contains our [RabbitMQ](https://www.rabbitmq.com/) image.

## Notes

- We use the ["streams"](https://www.rabbitmq.com/streams.html) protocol not only for performance and ease-of-use, but also because the [official Rust library](https://crates.io/crates/rabbitmq-stream-client) is based on the "streams" protocol
- Since we use "streams", we use non-standard ports, which are provided on the [upstream networking page](https://www.rabbitmq.com/networking.html)
- In our Docker Image, we enable the "streams" plugin by default, which is outlined in the [upstream DockerHub page](https://hub.docker.com/_/rabbitmq)
- The `-management` tagged upstream image enable the dashboard feature by default
- The `-alpine` tagged upstream images use [Alpine Linux](https://alpinelinux.org/) as the base image and offer size reduction