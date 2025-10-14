Tasks

- [x] Read format-version on push
- [x] make sure we use the asset name from metadata.json
- [ ] Qualifications: Deal with qualification input from the metadata file
- [ ] push management funcs
- [ ] Push code generators, also has the inputs on metadata.json
- [ ] Deal with existing func bindings for asset updates
- [ ] Deal with existing func names for new and updating assets
- [ ] Add push posthog event
- [x] Write an actual README file with run and build instructions

# si-conduit

This executable allows you to author schemas from your own local machine and
push them to your workspaces.

The only environment variable you need to set is `SI_API_KEY`, which is your API
key.

### To run:

```bash
deno task dev
```

Running the above without any arguments will print out the help text listing all
the avialable commands

### To build:

```bash
deno task build
```

This will build the executable `sdi-conduit` in the current directory.
