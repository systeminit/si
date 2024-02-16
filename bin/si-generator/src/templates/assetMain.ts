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

  const regionProp = new PropBuilder()
      .setKind("string")
      .setName("region")
      .setValueFrom(new ValueFromBuilder()
          .setKind("inputSocket")
          .setSocketName("Region")
          .build())
      .build();
  asset.addProp(regionProp);
<% } %>

  return asset.build();
}
`;
