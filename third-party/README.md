# Example Reindeer+Buck project

## Getting Started

The `setup.sh` script will build Reindeer, vendor some Cargo packages and
generate a BUCK file of build rules for them.

This will require a Rust installation (stable, but probably fairly recent), and
Buck to actually make use of the generated files.

You can learn more about Buck at [buck.build](https://buck.build). The
[getting started](https://buck.build/setup/getting_started.html) page should
help with getting it installed.

## Buck and its Configuration

The `.buckconfig` file both configures Buck and tells it where the top of the
"cell" is (current working tree). This only contains some very minimal Rust
configuration; most notable is overriding the default to Rust 2018.

(`.buck-java11` won't generally be needed.)

## Reindeer configuration

The files and directories Reindeer cares about are under `third-party/`:

- reindeer.toml - Reindeer's configuration. The directory containing this file
  is also the base for any relative paths mentioned in the file.
- Cargo.toml - You edit this to specify which packages you want to import, along
  with other settings like features, patches and so on, using the full syntax
  Cargo allows
- Cargo.lock - The resolved dependencies
- BUCK - The generated Buck build rules (once generated)
- .gitignore - This is used to ignore various inconvenient files in vendored
  code. Reindeer itself will look at this to edit them out of the vendored
  checksum.json files so that Cargo doesn't get upset.

In addition to these files, there are a few significant directories:

- vendor/ - where all the vendored code goes
- fixups/ - fixups tell Reindeer how to handle packages with build scripts and
  other special cases; most packages won't need anything here
- macros/ - Buck macros which map from the rules Reindeer generates to the
  actual build environment. This directory is not hard-coded and could be
  anywhere. The macros included here are quite minimal.
- top/ - Cargo needs a dummy package for the Cargo.toml (it doesn't allow a
  package which is _all_ dependencies)

## Project

The `project/` directory represents some end-user code. There's nothing notable
about it aside from its references to `//third-party:...` for its third-party
dependencies.

Once everything is set up, you should be able to build it with
`buck build //project:test` to build the executable or `buck run //project:test`
to just build and run in situ.
