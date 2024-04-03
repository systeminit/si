def _toolchain_alias_impl(ctx: AnalysisContext) -> list[Provider]:
    return ctx.attrs.actual.providers

toolchain_alias = rule(
    doc="""
toolchain_alias acts like alias but for toolchain rules.
The toolchain_alias itself is a toolchain rule and the `actual` argument is
expected to be a toolchain_rule as well.
    """,
    attrs={
        "actual": attrs.toolchain_dep(
            doc="The actual toolchain that is being aliased. This should be a toolchain rule."
        )
    },
    impl=_toolchain_alias_impl,
    is_toolchain_rule=True,
)
