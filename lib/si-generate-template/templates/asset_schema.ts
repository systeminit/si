function main() {
    const asset = new AssetBuilder();

    const namePrefix = new PropBuilder()
        .setName("Name Prefix")
        .setKind("string")
        .setDocumentation(`## Name Prefix
This string will be prepended to every component created by the template. It must be unique across every run of the template. For example, if you run the template with the Name Prefix 'dev-', then if you had a component named 'vpc',
your components will become 'dev-vpc'.

Running the template with the same Name Prefix more than once is an error, as it would duplicate existing infrastructure.
        `)
        .setValidationFormat(Joi.string().required())
        .build();
    asset.addProp(namePrefix);
{% if aws %}
    const extraProp = new PropBuilder()
        .setName("extra")
        .setKind("object")
        .setHidden(false)
        .setWidget(
            new PropWidgetDefinitionBuilder()
            .setKind("header")
            .build(),
        )
        .addChild(
            new PropBuilder()
            .setName("Region")
            .setKind("string")
            .setHidden(false)
            .setWidget(
                new PropWidgetDefinitionBuilder()
                .setKind("text")
                .build(),
            )
            .setValidationFormat(Joi.string().required())
            .suggestSource({
                schema: "Region",
                prop: "/domain/region"
            })
            .build(),
        )
        .build();
    asset.addProp(extraProp);

    const credSecretProp = new SecretPropBuilder()
        .setName("AWS Credential")
        .setSecretKind("AWS Credential")
        .build();
    asset.addSecretProp(credSecretProp);
{% endif %}
    return asset.build();
}
