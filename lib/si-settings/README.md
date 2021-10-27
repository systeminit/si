# Service Settings

This crate is a library for loading service settings. It contains all the
configuration for all the services, since a huge portion of it is all shared
and reptitive.

```rust
use si_settings::Settings;
let settings = Settings::new();
```

Eventually, this is going to be dope - because we will have the option to
either assemble all the GRPC services into a monolith, or deploy them
separately, or both.
