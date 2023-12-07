// @ts-nocheck

function main() {
  const implicitTypeSocket = new SocketDefinitionBuilder()
    .setName("A Ports")
    .setArity("many")
    .build();

  const explicitTypeSocket = new SocketDefinitionBuilder()
    .setName("B Ports")
    .setArity("many")
    .setType("port")
    .build();

  const nestedTypeSocket = new SocketDefinitionBuilder()
    .setName("C Ports")
    .setArity("many")
    .setType("Docker<Port<string>>")
    .build();

  return new AssetBuilder()
    .addOutputSocket(implicitTypeSocket)
    .addOutputSocket(explicitTypeSocket)
    .addOutputSocket(nestedTypeSocket)
    .build();
}
