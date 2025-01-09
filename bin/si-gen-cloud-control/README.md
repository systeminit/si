# Useful Link

* [Cloudformation resource type schema](https://docs.aws.amazon.com/cloudformation-cli/latest/userguide/resource-type-schema.html)

# Running tests

```sh
$ deno test -A
```

# Updating Schema

```sh
$ deno run -A main.ts fetch-schema
```

# Changing log output

```sh
$ env LOG_LEVEL=verbose deno test -A
```
