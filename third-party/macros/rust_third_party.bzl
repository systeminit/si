# @nolint

# vendored-date: 2023-05-03
# project: https://github.com/facebook/buck2
# commit-hash: 211b502350e2f15acc4a72b62ef602fc8091b8d7
# commit-date: 2023-05-03T11:44:33-0700
# source: https://github.com/facebook/buck2/blob/211b502350e2f15acc4a72b62ef602fc8091b8d7/shim/third-party/macros/rust_third_party.bzl

def third_party_rust_prebuilt_cxx_library(name, **kwargs):
    # FIXME: This should probably be a fixup.toml, but it currently can't be expressed.
    # The windows-sys crate does -lwindows to find windows. We pass libwindows.a on the command line,
    # which resolves the symbols, but the linker still needs to "find" windows, so we also put its
    # directory on the link options.
    if name.endswith("libwindows.a"):
        kwargs["exported_linker_flags"] = ["-Lshim/third-party/rust/" + kwargs["static_lib"].rpartition("/")[0]]

    native.prebuilt_cxx_library(name = name, **kwargs)
