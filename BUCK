load("//build-rules/ts.bzl", "pnpm_install")

alias(
    name = "council",
    actual = "//bin/council:council",
)

alias(
    name = "pinga",
    actual = "//bin/pinga:pinga",
)

alias(
    name = "sdf",
    actual = "//bin/sdf:sdf",
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
