# Checking ABI compliance

This contains some utilities for running `abi-compliance-checker` (which
you need to install). It will generate an HTML file in the `compat_reports`
directory which compares the ABI between different libcouchbase versions

## Running

Ensure you have the `abi-compliance-checker` package installed

Run `make`, defining these `make` variables:

* `OLDVER` - Name of the old version (for display purposes)
* `NEWVER` - Name of the new version (for display purposes)
* `OLDROOT` - Installation root of the old version, e.g. `/usr/local/libcouchbase-2.4.0`
* `NEWROOT` - Installation root of the new version, e.g. `/usr/local/libcouchbase-master`

e.g.

```
make \
    OLDVER=2.3.0 NEWVER=2.4.3 \
    OLDROOT=/sources/libcouchbase-2.3.0/inst NEWROOT=/sources/lcb-master/inst
```

Then open the generated HTML file in the `compat_reports` directory with your favorite
browser.
