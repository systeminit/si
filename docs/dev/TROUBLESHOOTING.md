# Troubleshooting

If re-running the [bootstrap script](../../scripts/bootstrap.sh) does not solve your issue
and you are certain that `main` is stable, this document may help with troubleshooting and debugging.

## Wiping the Slate Clean

Having trouble running SI or its tests? Want to go back to the beginning and
wipe the slate clean? Keeping in mind that this will erase your current
development database and clean all build artifacts, you can try this:

```bash
make clean prepare build
```

Where:

- `clean` removes all Cargo (Rust) build artifacts, and each TypeScript-based
  component will remove its `node_modules/` and associated build artifacts
- `prepare` brings down the supporting services running in a Docker Compose
  deployment (i.e. the PostgreSQL database and NATS), and
  then re-deploys them from scratch
- `build` builds all components, including apps, binaries, and
  libraries/packages

## Build and Runtime Errors on aarch64 (arm64)

For `aarch64 (arm64)` debugging, please refer to the aforementioned **Notes on aarch64 (arm64)** section.

## Seeing Errors Related to Procedural Macros

In your editor, you may find that you'll see errors like `"YourEnum does not implement Display"` if you are using
[`Display` from the `strum` crate](https://docs.rs/strum/latest/strum/derive.Display.html).
This is because your editor may not have proc (procedural) macros enabled.

As of 15 September 2022, this feature is not enabled in [IntelliJ Rust](https://www.jetbrains.com/rust/) by default and
can cause the [aforementioned issue](https://github.com/intellij-rust/intellij-rust/issues/8847) to occur (which
affects all [JetBrains](https://www.jetbrains.com/) IDEs, such as [CLion](https://www.jetbrains.com/clion/)).
Thus, you will have to use the experimental proc macros feature or wait for stable proc macros support.

## Hitting File Descriptor Limits in Integration Tests

Running all [dal integration tests](./lib/dal/tests/integration.rs) can result in hitting the file descriptor limit.
You may see NATS, Postgres and general failures containing the following message: "too many open files".
If this happens, you are likley hitting the file descriptors limit.

You can see all `ulimit` values by executing `ulimit -a`.
For your specific OS, please refer to its official documentation on how to increase the file descriptor limit
to a reasonable, stable, and likely-much-higher value.

### Setting the Limit with Persistence on macOS

While we recommend referring to the official documentation when possible, sometimes, it does not exist!
[This guide](https://becomethesolution.com/blogs/mac/increase-open-file-descriptor-limits-fix-too-many-open-files-errors-mac-os-x-10-14)
from an unofficial source may help persist file descriptor limit changes on macOS.
