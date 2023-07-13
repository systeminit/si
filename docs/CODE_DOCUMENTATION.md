# Code Documentation

This document contains all information related to developer documentation for this repository's source code.

## Quickstart

Let's say you want to learn what a `Component` or an `AttributeValue` is.
Where do you look?
You can generate and open the docs in your browser to find out!

```bash
buck2 run //lib/dal:doc -- --document-private-items --open
```

Our Rust crates contain module and documentation comments that can be generated into static webpages by `rustdoc`.
When in doubt, see if `doc` target for a Rust-based library has what you are looking for.

## How Do We Generate Rust Documentation?

As previously mentioned, for our Rust crates, we leverage `rustdoc` for seamless integration with
[IntelliJ Rust](https://www.jetbrains.com/rust/), [rust-analyzer](https://rust-analyzer.github.io/), and more.

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