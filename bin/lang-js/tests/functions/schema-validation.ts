// deno-lint-ignore-file
// @ts-nocheck

function main() {
  const propName = new PropBuilder()
    .setName("name")
    .setKind("string")
    .setWidget(
      new PropWidgetDefinitionBuilder().setKind("text")
        .build(),
    )
    .setValidationFormat(Joi.string().pattern(/^www\..*$/))
    .build();

  const propCount = new PropBuilder()
    .setName("count")
    .setKind("integer")
    .setWidget(
      new PropWidgetDefinitionBuilder().setKind("number")
        .build(),
    )
    .setValidationFormat(
      Joi.number().integer()
        .min(0)
        .max(2),
    )
    .build();

  const child1 = new PropBuilder()
    .setName("name")
    .setKind("string")
    .setWidget(
      new PropWidgetDefinitionBuilder().setKind("text")
        .build(),
    )
    .setValidationFormat(Joi.string().pattern(/^www\..*$/))
    .build();

  const child2 = new PropBuilder()
    .setName("name")
    .setKind("string")
    .setWidget(
      new PropWidgetDefinitionBuilder().setKind("text")
        .build(),
    )
    .setValidationFormat(Joi.number())
    .build();

  const parent = new PropBuilder()
    .setName("metadata")
    .setKind("object")
    .setWidget(
      new PropWidgetDefinitionBuilder().setKind("text")
        .build(),
    )
    .addChild(child1)
    .addChild(child2)
    .build();

  return new AssetBuilder()
    .addProp(propName)
    .addProp(propCount)
    .addProp(parent)
    .build();
}
