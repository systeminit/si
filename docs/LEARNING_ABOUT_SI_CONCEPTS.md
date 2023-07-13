# Learning About SI Concepts

As referenced in [CODE_DOCUMENTATION](./CODE_DOCUMENTATION.md), the `rustdoc` static webpages are an entrypoint
into learning about the Rust modules and structs that back many SI concepts.

Let's say you want to learn about what a `Component` is.
You may want to open the docs for the [dal](../lib/dal) as it is the center of many SI concepts.

You can generate and open the Rust documentation locally via the following command:

```bash
buck2 run lib/dal:doc -- --document-private-items --open
```