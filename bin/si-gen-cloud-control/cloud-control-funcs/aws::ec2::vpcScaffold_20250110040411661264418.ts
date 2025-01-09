function main() {
    const asset = new AssetBuilder();

    // Standard "extra" Props
    const extraProp = new PropBuilder()
        .setKind("object")
        .setName("extra")
        .addChild(
            new PropBuilder()
            .setKind("string")
            .setName("Region")
            .build(),
        )
        .addChild(
            new PropBuilder()
            .setKind("string")
            .setName("AwsResourceType")
            .setDefaultValue("AWS::EC2::VPC")
            .build(),
        )
        .addChild(
            new PropBuilder()
            .setKind("string")
            .setName("PrimaryIdentifier")
            .setDefaultValue("VpcId")
            .build(),
        )
        .addChild(
            new PropBuilder()
            .setName("AwsFieldMap")
            .setKind("map")
            .setEntry(
                new PropBuilder()
                .setName("AwsFieldMapCategory")
                .setKind("string")
                .build()
            )
            .build()
        )
        // Should have something that tracks which properties are create and write only
        .build();
    asset.addProp(extraProp);

    // Standard Sockets
    const credentialProp = new SecretPropBuilder()
        .setName("credential")
        .setSecretKind("AWS Credential")
        .build();
    asset.addSecretProp(credentialProp);

    const regionSocket = new SocketDefinitionBuilder()
        .setName("Region")
        .setArity("one")
        .build();
    asset.addInputSocket(regionSocket);

    // Create Only Properties
    const createOnlyProp = new PropBuilder()
        .setName("Create Only")
        .setKind("object");

    const cidrBlockProp = new PropBuilder()
        .setKind("string")
        .setName("CidrBlock")
        .setValidationFormat(Joi.string().required().regex(/^(([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])\.){3}([0-9]|[1-9][0-9]|1[0-9]{2}|2[0-4][0-9]|25[0-5])(\/)(1[6-9]|2[0-8])$/).messages({
            'string.pattern.base': 'Must be a valid IPv4 CIDR with CIDR Blocks between /16 and /28'
        }))
        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
        .build();
    createOnlyProp.addChild(cidrBlockProp);

    const ipv4IpamPoolIdProp = new PropBuilder()
        .setName("Ipv4IpamPoolId")
        .setKind("string")
        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
        .build();
    createOnlyProp.addChild(ipv4IpamPoolIdProp);

    const ipv4NetmaskLengthProp = new PropBuilder()
        .setName("Ipv4NetmaskLength")
        .setKind("string")
        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
        .build();
    createOnlyProp.addChild(ipv4NetmaskLengthProp);
    asset.addProp(createOnlyProp.build());

    // Create and Update Properties
    const updateableProp = new PropBuilder()
        .setName("Updateable")
        .setKind("object");

    const instanceTenancyProp = new PropBuilder()
        .setName("InstanceTenancy")
        .setKind("string")
        .build();
    updateableProp.addChild(instanceTenancyProp);

    const enableDnsSupportProp = new PropBuilder()
        .setName("EnableDnsSupport")
        .setKind("boolean")
        .build();
    updateableProp.addChild(enableDnsSupportProp);

    const enableDnsHostnamesProp = new PropBuilder()
        .setName("EnableDnsHostnames")
        .setKind("boolean")
        .build();
    updateableProp.addChild(enableDnsHostnamesProp);

    const tagsProp = new PropBuilder()
        .setName("Tags")
        .setKind("array")
        .setEntry(
            new PropBuilder()
            .setName("Tag")
            .setKind("object")
            .addChild(
                new PropBuilder()
                .setName("Key")
                .setKind("string")
                .build()
            )
            .addChild(
                new PropBuilder()
                .setName("Value")
                .setKind("string")
                .build()
            )
            .build()
        )
        .build();
    updateableProp.addChild(tagsProp);

    asset.addProp(updateableProp.build());

    // Read Only Properties
    const cidrBlockAssociationsProp = new PropBuilder()
        .setName("CidrBlockAssociations")
        .setKind("array")
        .setHidden(true)
        .setEntry(
            new PropBuilder()
            .setName("CidrBlockAssociation")
            .setKind("string")
            .build()
        )
        .build();
    asset.addResourceProp(cidrBlockAssociationsProp);
    const cidrBlockAssociationsSocket = new SocketDefinitionBuilder()
        .setName("CidrBlockAssociations")
        .setArity("one")
        .setConnectionAnnotation("cidrblockassociations<aws<array<string>>>")
        .build();
    asset.addOutputSocket(cidrBlockAssociationsSocket);

    const defaultNetworkAclProp = new PropBuilder()
        .setName("DefaultNetworkAcl")
        .setKind("string")
        .setHidden(true)
        .build();
    asset.addResourceProp(defaultNetworkAclProp);
    const defaultNetworkAclSocket = new SocketDefinitionBuilder()
        .setName("DefaultNetworkAcl")
        .setArity("one")
        .setConnectionAnnotation("defaultNetworkAcl<aws<string>>")
        .build();
    asset.addOutputSocket(defaultNetworkAclSocket);

    const defaultSecurityGroupProp = new PropBuilder()
        .setName("DefaultSecurityGroup")
        .setKind("string")
        .setHidden(true)
        .build();
    asset.addResourceProp(defaultSecurityGroupProp);
    const defaultSecurityGroupSocket = new SocketDefinitionBuilder()
        .setName("DefaultSecurityGroup")
        .setArity("one")
        .setConnectionAnnotation("defaultSecurityGroup<aws<string>>")
        .build();
    asset.addOutputSocket(defaultSecurityGroupSocket);

    const ipv6CidrBlocksProp = new PropBuilder()
        .setName("Ipv6CidrBlocks")
        .setKind("array")
        .setHidden(true)
        .setEntry(
            new PropBuilder()
            .setName("Ipv6CidrBlock")
            .setKind("string")
            .build()
        )
        .build();
    asset.addResourceProp(ipv6CidrBlocksProp);
    const ipv6CidrBlocksSocket = new SocketDefinitionBuilder()
        .setName("Ipv6CidrBlocks")
        .setArity("one")
        .setConnectionAnnotation("ipv6cidrblocks<aws<array<string>>>")
        .build();
    asset.addOutputSocket(ipv6CidrBlocksSocket);

    const vpcIdProp = new PropBuilder()
        .setName("VpcId")
        .setKind("string")
        .setHidden(true)
        .build();
    asset.addResourceProp(vpcIdProp)
    const vpcIdSocket = new SocketDefinitionBuilder()
        .setName("VpcId")
        .setArity("one")
        .setConnectionAnnotation("vpcId<aws<string>>")
        .build();
    asset.addOutputSocket(vpcIdSocket);

    return asset.build();
}
