# Documentation

This document contains all information related to developer documentation for the contents of this repository.

## How do we generate docs?

For Rust crates, we leverage `rustdoc` for seamless integration with `cargo doc`
, [IntelliJ Rust](https://www.jetbrains.com/rust/),
[rust-analyzer](https://rust-analyzer.github.io/), and more.

## Reading Rust Documentation

Build the docs for all of our crates and open the docs in your browser at [dal](./lib/dal) by executing
the following make target:

```bash
make docs-open
```

You can also execute the following:

```bash
cargo doc --open -p dal
```

If you would like to live-recompile docs while making changes on your development branch, you can execute the following
make target:

```bash
make docs-watch
```

> Please note: [cargo-watch](https://github.com/watchexec/cargo-watch) needs to be installed before using the above make
> target.
>
> ```bash
> cargo install --locked cargo-watch
> ```

## Writing Rust Documentation

We try to follow the
official ["How to write documentation"](https://doc.rust-lang.org/rustdoc/how-to-write-documentation.html) guide
from `rustdoc` as closely as possible.
Older areas of the codebase may not follow the guide and conventions derived from it.
We encourage updating older documentation as whilst navigating through SI crates.

### Additional Resources for Rust Documentation

* [RFC-1574](https://github.com/rust-lang/rfcs/blob/master/text/1574-more-api-documentation-conventions.md#appendix-a-full-conventions-text):
  more API documentation conventions for `rust-lang`
* ["Making Useful Documentation Comments" from "The Book"](https://doc.rust-lang.org/book/ch14-02-publishing-to-crates-io.html#making-useful-documentation-comments):
  a section of "The Book" covering useful documentation in the context of crate publishing