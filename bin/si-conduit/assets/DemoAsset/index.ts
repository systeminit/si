function main() {
  const asset = new AssetBuilder();

  const userNameProp = new PropBuilder()
    .setName("UserName")
    .setKind("string")
    .setValidationFormat(Joi.string().required())
    .setDocLink(
      "https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateUser.html#API_CreateUser_RequestParameters",
    )
    .setDocumentation(
      "The name of the user. Do not include the path in this value.\nIAM user, group, role, and policy names must be unique within the account. Names are not distinguished by case. For example, you cannot create resources named both `MyResource` and `myresource`.",
    )
    .build();
  asset.addProp(userNameProp);

  const pathProp = new PropBuilder()
    .setName("Path")
    .setKind("string")
    .setDocLink(
      "https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateUser.html#API_CreateUser_RequestParameters",
    )
    .setDocumentation(
      "The path to the group. For more information about paths, see [IAM identifiers](https://docs.aws.amazon.com/IAM/latest/UserGuide/Using_Identifiers.html) in the IAM User Guide.\nThis parameter is optional. If it is not included, it defaults to a slash (/).\nThis parameter allows (through its [regex pattern](http://wikipedia.org/wiki/regex)) a string of characters consisting of either a forward slash (/) by itself or a string that must begin and end with forward slashes. In addition, it can contain any ASCII character from the ! (\u0021 ) through the DEL character (\u007F ), including most punctuation characters, digits, and upper and lowercased letters.",
    )
    .build();
  asset.addProp(pathProp);

  const permissionsBoundaryProp = new PropBuilder()
    .setName("PermissionsBoundary")
    .setKind("string")
    .setDocLink(
      "https://docs.aws.amazon.com/IAM/latest/APIReference/API_CreateUser.html#API_CreateUser_RequestParameters",
    )
    .setDocumentation(
      "The ARN of the managed policy that is used to set the permissions boundary for the user.\nA permissions boundary policy defines the maximum permissions that identity-based policies can grant to an entity, but does not grant permissions. Permissions boundaries do not define the maximum permissions that a resource-based policy can grant to an entity. To learn more, see [Permissions boundaries for IAM entities](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies_boundaries.html) in the IAM User Guide.\nFor more information about policy types, see [Policy types](https://docs.aws.amazon.com/IAM/latest/UserGuide/access_policies.html#access_policy-types) in the IAM User Guide .",
    )
    .build();
  asset.addProp(permissionsBoundaryProp);

  const TagsProp = new PropBuilder()
    .setName("Tags")
    .setKind("array")
    .setHidden(false)
    .setWidget(
      new PropWidgetDefinitionBuilder()
        .setKind("array")
        .build(),
    )
    .setEntry(
      new PropBuilder()
        .setName("TagsItem")
        .setKind("object")
        .setHidden(false)
        .setWidget(
          new PropWidgetDefinitionBuilder()
            .setKind("header")
            .build(),
        )
        .addChild(
          new PropBuilder()
            .setName("Key")
            .setKind("string")
            .setHidden(false)
            .setWidget(
              new PropWidgetDefinitionBuilder()
                .setKind("text")
                .build(),
            )
            .setValidationFormat(Joi.string().required())
            .setDocLink(
              "https://docs.aws.amazon.com/IAM/latest/APIReference/API_Tag.html",
            )
            .setDocumentation("The tag key.")
            .build(),
        )
        .addChild(
          new PropBuilder()
            .setName("Value")
            .setKind("string")
            .setHidden(false)
            .setWidget(
              new PropWidgetDefinitionBuilder()
                .setKind("text")
                .build(),
            )
            .setValidationFormat(Joi.string().required())
            .setDocLink(
              "https://docs.aws.amazon.com/IAM/latest/APIReference/API_Tag.html",
            )
            .setDocumentation("The tag value.")
            .build(),
        )
        .build(),
    )
    .build();
  asset.addProp(TagsProp);

  const AWSCredentialSecretProp = new SecretPropBuilder()
    .setName("AWS Credential")
    .setSecretKind("AWS Credential")
    .build();
  asset.addSecretProp(AWSCredentialSecretProp);

  const identityType = new PropBuilder()
    .setName("IdentityType")
    .setKind("string")
    .setDefaultValue("user")
    .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
    .setHidden(true)
    .build();
  asset.addProp(identityType);

  const arnResource = new PropBuilder()
    .setName("Arn")
    .setKind("string")
    .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
    .build();
  asset.addResourceProp(arnResource);

  return asset.build();
}
