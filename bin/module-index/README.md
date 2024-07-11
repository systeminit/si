# `module-index`

This document contains information related to running the `module-index`.

## Running Locally with the Full Stack

1. Export valid AWS credentials in your environment (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY` and `AWS_SESSION_TOKEN`)
1. Modify `app/web/.env` to use the following line: `VITE_MODULE_INDEX_API_URL=http://localhost:5157` _(do not commit this change!)_
1. Run the following command: `SI_MODULE_INDEX_URL=http://localhost:5157 buck2 run dev:up`
1. Navigate to the Tilt dashboard
1. Observe that `sdf` will fail on first boot because our local `module-index` isn't running
1. Run `module-index` in the Tilt dashboard
1. Restart `sdf` and observe that it starts successfully

If you are editing source code, you can restart all core services, as usual.

> [!NOTE]
> Running `module-index` locally does not yet work with [LocalStack](https://github.com/localstack/localstack) since credential validation requires multiple `AWS_*` variables and not just `AWS_ENDPOINT`.
