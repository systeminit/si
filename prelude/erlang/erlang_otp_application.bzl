# Copyright (c) Meta Platforms, Inc. and affiliates.
#
# This source code is licensed under both the MIT license found in the
# LICENSE-MIT file in the root directory of this source tree and the Apache
# License, Version 2.0 found in the LICENSE-APACHE file in the root directory
# of this source tree.

load(":erlang_info.bzl", "ErlangAppInfo")

# This is a superset of all available OTP applications and needs to be manually updated
# if new applications make it into OTP. New applications will not be available until
# they are listed here.
otp_applications = [
    "stdlib",
    "sasl",
    "kernel",
    "compiler",
    "tools",
    "common_test",
    "runtime_tools",
    "inets",
    "parsetools",
    "xmerl",
    "edoc",
    "erl_docgen",
    "snmp",
    "erl_interface",
    "asn1",
    "jinterface",
    "wx",
    "debugger",
    "reltool",
    "mnesia",
    "crypto",
    "os_mon",
    "syntax_tools",
    "public_key",
    "ssl",
    "observer",
    "diameter",
    "et",
    "megaco",
    "eunit",
    "ssh",
    "eldap",
    "dialyzer",
    "ftp",
    "tftp",
    "erts",
]

def gen_otp_applications() -> None:
    for name in otp_applications:
        _erlang_otp_application_rule(name = name, version = "dynamic", visibility = ["PUBLIC"])
    return None

def normalize_application(name: str) -> str:
    """Try to translate OTP application names to internal targets so users can write
    `kernel` instead of `prelude//erlang/applications:kernel`
    """
    if ":" not in name:
        if name in otp_applications:
            return "prelude//erlang/applications:{}".format(name)
        else:
            fail('Unknown OTP application "{app}". If this is not supposed to be an OTP application,  did you mean ":{app}"?'.format(app = name))

    return name

def _erlang_otp_application_impl(ctx: AnalysisContext) -> list[Provider]:
    """virtual OTP application for referencing only
    """
    return [
        DefaultInfo(),
        ErlangAppInfo(
            name = ctx.attrs.name,
            version = ctx.attrs.version,
            beams = [],
            includes = [],
            dependencies = {},
            start_dependencies = None,
            include_dir = None,
            virtual = True,
            app_folder = None,
        ),
    ]

_erlang_otp_application_rule = rule(
    impl = _erlang_otp_application_impl,
    attrs = {
        "version": attrs.string(),
    },
)
