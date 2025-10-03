function main() {
  const regionProp = new PropBuilder()
    .setName("region")
    .setKind("string")
    .build();


  const credentialProp = new SecretPropBuilder()
    .setName("credential")
    .setSecretKind("AWS Credential")
    .build();

  return new AssetBuilder()
    .addProp(regionProp)
    .addSecretProp(credentialProp)
    .build();
}
