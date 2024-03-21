load(
    "@prelude-si//:e2e.bzl",
    _e2e = "e2e",
    _e2e_test = "e2e_test",
)

def e2e(
        name,
        **kwargs):
    _e2e(
        name = name,
        **kwargs,
    )
    _e2e_test(
        name = name,
        **kwargs,
    )