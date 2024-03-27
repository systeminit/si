load(
    "@prelude-si//:e2e.bzl",
    _e2e_test = "e2e_test",

)

def e2e_test(
        name,
        visibility = ["PUBLIC"],
        **kwargs):
    _e2e_test(
        name = name,
        visibility = visibility,
        **kwargs
    )
