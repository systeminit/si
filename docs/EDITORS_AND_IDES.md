# Editors and IDEs

This document contains information related to using editors and IDEs when developing the System Initiative software.

## Seeing Errors Related to Procedural Macros

In your editor, you may find that you'll see errors like `"YourEnum does not implement Display"` if you are using
[`Display` from the `strum` crate](https://docs.rs/strum/latest/strum/derive.Display.html).
This is because your editor may not have proc (procedural) macros enabled.

As of 15 September 2022, this feature is not enabled in [IntelliJ Rust](https://www.jetbrains.com/rust/) by default and
can cause the [aforementioned issue](https://github.com/intellij-rust/intellij-rust/issues/8847) to occur (which
affects all [JetBrains](https://www.jetbrains.com/) IDEs, such as [CLion](https://www.jetbrains.com/clion/)).
Thus, you will have to use the experimental proc macros feature or wait for stable proc macros support.

## Direnv

For notes on using plugins with `direnv`, see [`DEVELOPMENT_ENVIRONMENT`](./DEVELOPMENT_ENVIRONMENT.md).