# `module-index`

This document contains information related to running the `module-index`.

## Running Locally with the Full Stack

1. Run the system as usual, whether it's running locally or on a remote server via `buck2 run dev:up` (refer to the [DOCS](../../DOCS.md) for more information on remote development)
1. Wait until all migrations succeed for `sdf` and all services are running and then CTRL+C
1. _DO NOT_ run `buck2 run dev:down` as we want the goodies from the initial setup
1. Export valid AWS credentials in your environment (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` and `AWS_SESSION_TOKEN`)
1. Run your `buck2 run dev:up` command (either local or remote) with two new environment variables: `VITE_MODULE_INDEX_API_URL=http://<localhost-or-remote-instance-hostname-here>:5157` and `SI_MODULE_INDEX_URL=http://localhost:5157`
1. Navigate to the Tilt dashboard
1. Observe that `sdf` will fail on first boot because our local `module-index` isn't running
1. Run `module-index` in the Tilt dashboard
1. Restart `sdf` and observe that it starts successfully

## Testing Changes Related to Modules

First, ensure that you have a local `module-index` running with the steps above.
Now, create a second local workspace for your users.
With two local workspaces, you can have two browser windows open (one per workspace) and test uploading, installing and upgrading modules.
