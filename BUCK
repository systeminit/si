load("//build-rules/ts.bzl", "pnpm_install")

alias(
    name = "council",
    actual = "//bin/council:council",
)

alias(
    name = "veritech",
    actual = "//bin/veritech:veritech",
)

alias(
    name = "prepare",
    actual = "//component/deploy:prepare",
)

alias(
    name = "down",
    actual = "//component/deploy:down",
)

pnpm_install(
    name = "pnpm-install",
    srcs = [
        "//bin/lang-js:package.json"
    ],
    visibility = ["PUBLIC"]
)
