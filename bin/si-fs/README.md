# si-fs

A FUSE filesystem for editing System Initiative assets in your favorite editor.

1. Get your System Initiative bearer token:
   - Head to https://auth.systeminit.com/workspaces
   - Pick the workspace you want to mount as a fuse filesystem.
   - Click the settings "gear" icon and choose the API Tokens option.
   - Generate an API token for use with the fuse filesystem.
   - Add the token to your env with `export SI_BEARER_TOKEN='<the token here>'` (the single quotes will ensure the base64 parses on your command line).

2. Mount the filesystem:

```bash
buck2 run //bin/si-fs -- --foreground --workspace-id <WORKSPACE_ID> [--endpoint <ENDPOINT>] /mountpoint
```

At this point, we recommend running in the foreground, so you can see any errors
returned by the API client. In the future we will improve telemetry and provide
a file on the filesystem that streams errors and warnings.

3. You can now browse and edit assets at `/mountpoint`

## Features

- Browse assets in a filesystem hierarchy
- Edit assets with your preferred editor
- Unlock and install assets
- Create new assets and functions
- Edit asset bindings
- Changes sync on write to System Initiative

## Commands

### Create a change set

To create a new change set to work in, do:

```
cd /mountpoint/change-sets;
mkdir new_change_set_name
```

A change set will be forked from `HEAD` in that directory.

### Install a schema

To install an uinstalled schema, run:

```
cd /mountpoint/change-sets/<YOUR CHANGE SET>/schemas/<YOUR SCHEMA>;
touch INSTALLED
```

On the next read of the schema directory, you will see the `definition` and `functions` folders.

### Unlock a schema

To unlock a schema, create the `unlocked` directory in the schema's definition directory:

```
cd /mountpoint/change-sets/<YOUR CHANGE SET>/schemas/<YOUR SCHEMA>/definition;
mkdir unlocked
```

### Unlock a function

To unlock a function, create the `unlocked` directory in the function's definition directory:

```
cd /mountpoint/change-sets/<YOUR CHANGE SET>/schemas/<YOUR SCHEMA>/functions/<FUNCTION KIND>/<FUNCTION NAME>;
mkdir unlocked
```

Unlocking a function will automatically unlock the schema that function is attached to.

NOTE: Only functions in the schema directory can be unlocked. Functions in the
change-set functions folder are read-only.

#### Create a new function for a schema

You can make a new function with `mkdir`.

```
cd /mountpoint/change-sets/<YOUR CHANGE SET>/schemas/<YOUR SCHEMA>/functions/<FUNCTION KIND>/;
mkdir new_func_name
```

Authorization, Qualification, Code Generation and Management functions will be
created with default bindings and automatically attached to the unlocked schema.

Attribute and Action functions will be in a pending state. Edit the
`PENDING_BINDINGS_EDIT_ME.json` file to configure the initial bindings for these
functions. If the bindings parse and are accepted by the backend, the pending
bindings file will disappear, and an `unlocked` folder will appear in their
place, containing the initial function code and other attributes.

### Attach an existing function

An existing function can be attached with `mv`.

Find the function you want to attach in another schemas functions folder, or in
the change-set level functions folder.

Then cd to the function kind folder for the schema you want to attach that
function to.

If attaching a function from another schema, do the following:

```
cd /mountpoint/change-sets/<CHANGE SET>/schemas/<YOUR SCHEMA>/functions/<FUNCTION KIND>
mv  /mountpoint/change-sets/<CHANGE SET>/schemas/<OTHER SCHEMA>/functions/<FUNCTION KIND>/<FUNCTION NAME>/locked .
```

The `unlocked` can also be attached.

If attaching a function from the change-set level functions folder:

```
cd /mountpoint/change-sets/<CHANGE SET>/schemas/<YOUR SCHEMA>/functions/<FUNCTION KIND>
mv  /mountpoint/change-sets/<CHANGE SET>/functions/<FUNCTION KIND>/<FUNCTION NAME> .
```

The function must be on the same change set as the schema you are attaching to.

Like when creating a new function, Attribute and Action functions will be put
into a pending state.

## Mac Setup

Ensure you setup macFUSE before attempting above setup
 https://github.com/macfuse/macfuse/wiki/Getting-Started

You will need to use a mountpoint in your user profile otherwise you will get `std io error: Read-only file system (os error 30)` when trying to write to the filesystem.

eg. `buck2 run //bin/si-fs -- --foreground --workspace-id <WORKSPACE_ID> ~/si-mount`

