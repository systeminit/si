# Troubleshooting

This document contains common troubleshooting scenarios when working on the System Initiative software.

## Build Errors Related to Running Services Reliant on `node_modules`

Since we switched to `buck2` for our build infrastructure in mid-2023, you may experience issues when running services reliant on `node_modules` within older cloned instances of the repostiory.
To solve these build errors, run the following in the root of your repository:

> *Warning: this command deletes files.*
> Ensure your current working directory is the root of the repository and understand what the command does before executing.
> Please reach out to us [on Discord](https://discord.com/invite/system-init) if you have any questions.

```bash
find app bin lib third-party/node -type d -name node_modules -exec rm -rf {} \;; rm -rf node_modules
```

## NATS Jetstream Not Enabled

If you see an error related to [NATS Jetstream](https://docs.nats.io/nats-concepts/jetstream) not being enabled when running the stack or tests, your local [`systeminit/nats`](https://hub.docker.com/repository/docker/systeminit/nats/) image is likely out of date.
To get the most up-to-date images (including the aforementioned image), run the following command:

```bash
buck2 run //dev:pull
```

## Potential Service Conflicts

SI uses external services in conjunction with its native components.
These external services are deployed via [`docker compose`](https://docs.docker.com/compose/) and are configured to stick to their default settings as closely as possible, including port settings.
Thus, it is worth checking if you are running these services to avoid conflicts when running SI.
Potentially conflicting services include, but are not limited to, the following:

* PostgreSQL DB
* OpenTelemetry
* NATS
* Watchtower

In the case of a port conflict, a good strategy is to temporarily disable the host service until SI is no longer being run.

## Seeing Errors Related to Procedural Macros

In your editor, you may find that you'll see errors like `"YourEnum does not implement Display"` if you are using [`Display` from the `strum` crate](https://docs.rs/strum/latest/strum/derive.Display.html).
This is because your editor may not have proc (procedural) macros enabled.
Check out your editor or relevant plugin docs for more information.