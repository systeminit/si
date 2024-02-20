export const partial = `
function main() {
  const asset = new AssetBuilder();

<% for (const prop of it.props) { %>
<%~ include("@renderPropPartial", { prop, omitVariable: false }) %>
<% } %>

<% if (it.provider == "aws") { %>
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
        .setValueFrom(new ValueFromBuilder()
            .setKind("inputSocket")
            .setSocketName("Region")
            .build()
        ).build()
      )
      .build();

  asset.addProp(extraProp);
<% } %>

  return asset.build();
}
`;
