# Clover

Clover generates full AWS schema coverage for all resource types in the
[Cloudformation schema](https://docs.aws.amazon.com/cloudformation-cli/latest/userguide/resource-type-schema.html).
It may be used to generate schemas for other providers in the future.

## Quickstart

This guide is for using generated Clover schemas for a local dev SI instance.

1. Run an instance of SI locally. See the [README](../../README.md) for more
   details.
1. In your local workspace, find the bearer token. You can find it in the
   headers of any request made to the SI backend and it starts with "Bearer". Or
   you can make and use a Workspace token following
   [this guide](https://docs.systeminit.com/reference/workspaces#generate-api-token).
1. Copy the token to a safe location and remove the leading "Bearer" portion.
1. Open your terminal for the next steps.
1. Make your bearer token available for clover:
   `export SI_BEARER_TOKEN=<your-token>`.
1. Change to the clover directory: `cd bin/clover`.
1. Initialize the git submodule:
   `git submodule init src/provider-schemas/azure-rest-api-specs`.
1. Change to the si-specs directory: `cd si-specs`.
1. Generate clover schemas using `deno`. For this guide, we'll use Azure:
   `deno task run generate-specs --provider=azure`. The directory will contain
   all generated Azure schemas.
1. In your local workspace in your browser, navigate to the customization
   screen.
1. On the upper left, switch to the "MODULES (INTERNAL)" tab and click the
   upload to cloud button. You'll be prompted to choose which files to upload.
   The file names correspond with the modules that will be uploaded. You can
   find the generated JSON files in `bin/clover/si-specs`
1. In either a newly created change set or your non-HEAD change set, you will
   now be able to create components with your Azure module(s)!

> [!TIP]
> If you have issues generating schemas, you may need to ensure you are using
> your local module index or that your submodule need to be re-configured.
>
> For the former, run the following command: `unset SI_MODULE_INDEX_URL`. This
> ensures that clover will use default settings for generation.
>
> For the latter, you may need to update the submodule with
> `git submodule update` or to de-initialize the submodule and start over via
> `git submodule deinit -f src/provider-schemas/azure-rest-api-specs` (assuming
> you are using Azure).

## Initial Setup for Development

Before you can run the schema generator, you must download the latest
[Cloudformation resource type schema](https://docs.aws.amazon.com/cloudformation-cli/latest/userguide/resource-type-schema.html)
to cloudformation-schema/*.json:

```shell
deno task run fetch-schema
```

## Generating Schemas

This will fetch the current ID for each schema, and then regenerate all schemas
into the si-specs directory.

```shell
deno task run generate-specs [...SCHEMA]
```

- `generate-specs` with no arguments to regenerate all specs
- `generate-specs EC2 S3 SecurityGroup` to regenerate all EC2 and S3 schemas,
  plus the SecurityGroup schema
- `SI_MODULE_INDEX_URL=https://module-index.systeminit.com` to point at
  production
- `SI_MODULE_INDEX_URL=http://127.0.0.1:5157` to point at your local module
  index. (**If you do this, do NOT upload the resulting specs to
  production--they will have the wrong module IDs.**)
- `LOG_LEVEL=debug` (or info, warning, error) for log output

### Diffing Your Changes

When you have changed heuristics, you will want to look at the differences--but
the specs contain ULIDs, which change each time. You can run
`anonymize-specs.sh` runs jq to remove these (which will not change the specs,
but create an anonymized copy in si-specs/anonymized). After that, you can diff
the anonymized specs to see what your heuristic affected.

1. **Baseline**: Before you run your new changes, generated an anonymized
   baseline in `si-specs-old/anonymized`:

   ```sh
   $ deno task run generate-specs && ./anonymize-specs.sh
   $ cp -R si-specs si-specs-old
   ```

2. **Regenerate**: Regenerate and anonymize the specs to `si-specs/anonymized`:

   ```sh
   $ deno task run generate-specs && ./anonymize-specs.sh
   ```

3. **Diff**: Diff the results:

   ```sh
   diff -u si-specs-old/anonymized si-specs/anonymized
   ```

4. Repeat steps 2-3 until satisfied.

### Note about Schema Code changes

If you are changing a lot of schema functions themselves, every package will
show `codeBase64` changed, which is a giant blob and can get overwhelming.

To exclude codeBase64, add this to anonymize-specs.sh:

```shell
./anonymize-specs.sh --remove-props ".funcs[].data.codeBase64"
```

Just note that if you do this, you won't know which modules had their schema
code changed! Don't run it all the time.

## Running tests

```shell
buck2 run //bin/clover:test-unit
```

### Changing log output

```shell
env LOG_LEVEL=verbose deno test --unstable-sloppy-imports -A
```
