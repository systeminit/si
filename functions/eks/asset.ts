function main() {
    const asset = new AssetBuilder();

    const nameProp = new PropBuilder()
        .setName("name")
        .setValidationFormat(Joi.string().required().regex(/^[a-zA-Z0-9][a-zA-Z0-9_-]{0,99}$/).messages({
            'string.pattern.base': 'The name can contain only alphanumeric characters (case-sensitive), hyphens, and underscores. It must start with an alphanumeric character and can’t be longer than 100 characters.'
        }))
        .setKind("string").build();
    asset.addProp(nameProp);

    const clusterNameSocket = new SocketDefinitionBuilder()
        .setName("Cluster Name")
        .setArity("many")
        .build();
    asset.addOutputSocket(clusterNameSocket);

    const versionProp = new PropBuilder()
        .setName("version")
        .setKind("string")
        .build();
    asset.addProp(versionProp);

    const roleArnProp = new PropBuilder()
        .setName("roleArn")
        .setKind("string")
        .build();
    asset.addProp(roleArnProp);

    const roleArnSocket = new SocketDefinitionBuilder()
        .setName("Role ARN")
        .setConnectionAnnotation("ARN")
        .setArity("one")
        .build();
    asset.addInputSocket(roleArnSocket);

    const resourcesVpcConfigProp = new PropBuilder()
        .setName("resourcesVpcConfig")
        .setKind("object")
        .addChild(
            new PropBuilder()
                .setName("subnetIds")
                .setKind("array")
                .setEntry(
                    new PropBuilder().setName("subnetIdsChild").setKind("string").build(),
                )
                .build(),
        )
        .addChild(
            new PropBuilder()
                .setName("securityGroupIds")
                .setKind("array")
                .setEntry(
                    new PropBuilder()
                        .setName("securityGroupIdsChild")
                        .setKind("string")
                        .build(),
                )
                .build(),
        )
        .addChild(
            new PropBuilder()
                .setName("endpointPublicAccess")
                .setKind("boolean")
                .build(),
        )
        .addChild(
            new PropBuilder()
                .setName("endpointPrivateAccess")
                .setKind("boolean")
                .build(),
        )
        .addChild(
            new PropBuilder()
                .setName("publicAccessCidrs")
                .setKind("array")
                .setEntry(
                    new PropBuilder()
                        .setName("publicAccessCidrsChild")
                        .setKind("string")
                        .build(),
                )
                .build(),
        )
        .build();
    asset.addProp(resourcesVpcConfigProp);

    const securityGroupSocket = new SocketDefinitionBuilder()
        .setArity("many")
        .setName("Security Group ID")
        .build();
    asset.addInputSocket(securityGroupSocket);

    const subnetIdSocket = new SocketDefinitionBuilder()
        .setName("Subnet ID")
        .setArity("many")
        .build();
    asset.addInputSocket(subnetIdSocket);

    const kubernetesNetworkConfigProp = new PropBuilder()
        .setName("kubernetesNetworkConfig")
        .setKind("object")
        .addChild(
            new PropBuilder().setName("serviceIpv4Cidr").setKind("string").build(),
        )
        .addChild(new PropBuilder().setName("ipFamily").setKind("string").build())
        .build();
    asset.addProp(kubernetesNetworkConfigProp);

    const enabledLoggingTypes = new PropBuilder()
        .setName("enabledLoggingTypes")
        .setKind("array")
        .setEntry(new PropBuilder()
            .setName("EnabledLoggingType")
            .setKind("string")
            .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
            .build())
        .build();
    asset.addProp(enabledLoggingTypes)

    const tagsProp = new PropBuilder()
        .setName("tags")
        .setKind("map")
        .setEntry(new PropBuilder().setName("tag").setKind("string").build())
        .build();
    asset.addProp(tagsProp);

    const encryptionConfigProp = new PropBuilder()
        .setName("encryptionConfig")
        .setKind("object")
        .addChild(
            new PropBuilder()
                .setName("resources")
                .setKind("array")
                .setEntry(
                    new PropBuilder()
                        .setName("resourcesChild")
                        .setKind("string")
                        .build(),
                )
                .build(),
        )
        .addChild(
            new PropBuilder()
                .setName("provider")
                .setKind("object")
                .addChild(
                    new PropBuilder().setName("keyArn").setKind("string").build(),
                )
                .build(),
        )
        .build()
    asset.addProp(encryptionConfigProp);

    const kmsKeyArnSocket = new SocketDefinitionBuilder()
        .setName("Key ARN")
        .setArity("one")
        .build();
    asset.addInputSocket(kmsKeyArnSocket);

    const accessConfigProp = new PropBuilder()
        .setName("accessConfig")
        .setKind("object")
        .addChild(
            new PropBuilder()
                .setName("bootstrapClusterCreatorAdminPermissions")
                .setKind("boolean")
                .build(),
        )
        .addChild(
            new PropBuilder()
                .setName("authenticationMode")
                .setWidget(new PropWidgetDefinitionBuilder()
                    .setKind("select")
                    .addOption("API", "API")
                    .addOption("API_AND_CONFIG_MAP", "API_AND_CONFIG_MAP")
                    .addOption("CONFIG_MAP", "CONFIG_MAP")
                    .build())
                .setKind("string")
                .build(),
        )
        .build();
    asset.addProp(accessConfigProp);

    const credentialProp = new SecretPropBuilder()
        .setName("credential")
        .setSecretKind("AWS Credential")
        .build();
    asset.addSecretProp(credentialProp);

    const regionSocket = new SocketDefinitionBuilder()
        .setArity("one")
        .setName("Region")
        .build();
    asset.addInputSocket(regionSocket);

    // Add any props needed for information that isn't
    // strictly part of the object domain here.
    const extraProp = new PropBuilder()
        .setKind("object")
        .setName("extra")
        .addChild(
            new PropBuilder()
                .setKind("string")
                .setName("Region")
                .build(),
        )
        .build();

    asset.addProp(extraProp);

    // Resource Props
    const clusterArn = new PropBuilder()
        .setName("arn")
        .setKind("string")
        .setHidden(true)
        .build();
    asset.addResourceProp(clusterArn);

    const clusterName = new PropBuilder()
        .setName("name")
        .setKind("string")
        .setHidden(true)
        .build();
    asset.addResourceProp(clusterName);

    const clusterEndpoint = new PropBuilder()
        .setName("endpoint")
        .setKind("string")
        .setHidden(true)
        .build();
    asset.addResourceProp(clusterEndpoint);

    const certificateAuthority = new PropBuilder()
        .setName("certificateAuthority")
        .setKind("object")
        .setHidden(true)
        .addChild(
            new PropBuilder()
                .setName("data")
                .setKind("string")
                .setHidden(true)
                .build(),
        )
        .build();
    asset.addResourceProp(certificateAuthority);

    // Cluster ARN Output Socket
    const clusterArnSocket = new SocketDefinitionBuilder()
        .setArity("many")
        .setName("Cluster ARN")
        .build();
    asset.addOutputSocket(clusterArnSocket);

    const clusterCaData = new SocketDefinitionBuilder()
        .setArity("many")
        .setName("Cluster CA Data")
        .build();
    asset.addOutputSocket(clusterCaData);

    const clusterEndpointSocket = new SocketDefinitionBuilder()
        .setArity("many")
        .setName("Cluster Endpoint")
        .build();
    asset.addOutputSocket(clusterEndpointSocket);

    return asset.build();
}