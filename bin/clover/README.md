clover generates full AWS asset coverage for all resource types in the
[Cloudformation schema](https://docs.aws.amazon.com/cloudformation-cli/latest/userguide/resource-type-schema.html).

# Initial Setup

Before you can run the schema generator, you must download the latest
[Cloudformation resource type schema](https://docs.aws.amazon.com/cloudformation-cli/latest/userguide/resource-type-schema.html)
to cloudformation-schema/*.json:

```sh
$ deno task run fetch-schema
```

# Generating Assets

This will fetch the current ID for each schema, and then regenerate all schemas
into the si-specs directory.

```sh
$ deno task run generate-specs [...SCHEMA]
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

## Diffing Your Changes

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

## Note about Asset Code changes

If you are changing a lot of asset functions themselves, every package will show
`codeBase64` changed, which is a giant blob and can get overwhelming.

To exclude codeBase64, add this to anonymize-specs.sh:

```sh
$ ./anonymize-specs.sh --remove-props ".funcs[].data.codeBase64"
```

Just note that if you do this, you won't know which modules had their asset code
changed! Don't run it all the time.

# Running tests

```sh
$ buck2 run //bin/clover:test-unit
```

## Changing log output

```sh
$ env LOG_LEVEL=verbose deno test --unstable-sloppy-imports -A
```
