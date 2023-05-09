load("//build-rules/ts.bzl", "pnpm_install")

alias(
    name = "council",
    actual = "//bin/council:council",
)

alias(
    name = "veritech",
    actual = "//bin/veritech:veritech",
)

pnpm_install(
    name = "pnpm-install",
    srcs = [
        "//bin/lang-js:package.json"
    ],
    visibility = ["PUBLIC"]
)
