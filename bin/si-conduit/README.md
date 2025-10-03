# si-conduit

This executable allows you to author schemas from your own local machine and
push them to your workspaces.

The only environment variable you need to set is `SI_API_KEY`, which is your API
key.

If you want to override the API endpoint, you can set the `SI_API_URL` environment variable.
Otherwise, it will default to `https://api.systeminit.com`.

### To run:

```bash
deno task dev
```

Running the above without any arguments will print out the help text listing all
the available commands

### To build from source tree:

```bash
deno task build
```

This will build the executable `si-conduit` in the current directory.

### To build remotely:

```bash
deno compile \
  --allow-all \
  --reload \
  --output=si-conduit \
  --import-map=https://raw.githubusercontent.com/systeminit/si/main/bin/si-conduit/deno.json \
  https://raw.githubusercontent.com/systeminit/si/main/bin/si-conduit/main.ts
```

This will build the executable `si-conduit` in the current directory.

### To use:
Start by defining an asset folder where you'll keep your asset definitions. They will be subfolders of the asset folder.
Conduit is smart enough to create it for you if you specify a path that does not exist yet.


#### Create a template of a schema to modify:
```bash
si-conduit scaffold $asset_name -f $asset-folder
```

#### Push the assets on a folder:
```bash
si-conduit push $asset-folder # append -s to skip the confirmation prompt
```


### TODO:

- [x] Read format-version on push
- [x] make sure we use the asset name from metadata.json
- [x] ~~Qualifications: Deal with qualification input from the metadata file~~
- [x] push management funcs
- [x] Push code generators ~~, also has the inputs on metadata.json~~
- [ ] Deal with existing func bindings for asset updates
- [ ] Deal with existing func names for new and updating assets
- [x] Add push posthog event
- [x] Write an actual README file with run and build instructions

