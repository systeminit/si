load(
    "@prelude-si//macros:native.bzl",
    _alias = "alias",
)
alias = _alias

load(
    "@prelude-si//macros:rust.bzl",
    _rust_binary = "rust_binary",
    _rust_library = "rust_library",
)
rust_binary = _rust_binary
rust_library = _rust_library
