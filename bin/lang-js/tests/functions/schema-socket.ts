// deno-lint-ignore-file
// @ts-nocheck

function main() {
  const implicitTypeSocket = new SocketDefinitionBuilder()
    .setName("ARN A")
    .setArity("many")
    .build();

  const explicitTypeSocket = new SocketDefinitionBuilder()
    .setName("ARN B")
    .setArity("many")
    .setConnectionAnnotation("ARN")
    .build();

  const nestedTypeSocket = new SocketDefinitionBuilder()
    .setName("ARN C")
    .setArity("many")
    .setConnectionAnnotation("ARN<string>")
    .setConnectionAnnotation("Amazon Resource Name<string>")
    .build();

  return new AssetBuilder()
    .addOutputSocket(implicitTypeSocket)
    .addOutputSocket(explicitTypeSocket)
    .addInputSocket(nestedTypeSocket)
    .build();
}
