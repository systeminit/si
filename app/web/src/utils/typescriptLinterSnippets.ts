import { snippetCompletion } from "@codemirror/autocomplete";

export const snippets = [
  // new prop
  snippetCompletion(
    `const \${propName} = new PropBuilder()
      .setName("\${name}")
      .setKind("\${string}")
      .setWidget(new PropWidgetDefinitionBuilder().setKind("\${text}")
      .build())
    .build();`,
    {
      label: "New Prop Snippet",
      type: "function",
    },
  ),

  // new secret prop
  snippetCompletion(
    `const \${secretPropName} = new SecretPropBuilder()
        .setName("\${name}")
        .setSecretKind("\${Secret Kind}")
        .build();`,
    {
      label: "New Secret Prop Snippet",
      type: "function",
    },
  ),

  // new secret definition
  snippetCompletion(
    `const \${secretDefinition} = new SecretDefinitionBuilder()
        .setName("\${name}")
        .addProp(
            new PropBuilder()
            .setName("\${value}")
            .setKind("\${string}")
            .setWidget(
                new PropWidgetDefinitionBuilder()
                .setKind("password")
                .build()
            ).build()
        ).build();`,
    {
      label: "New Secret Definition",
      type: "function",
    },
  ),

  // basic socket
  snippetCompletion(
    `const \${socketName} = new SocketDefinitionBuilder()
      .setName("\${name}")
      .setArity("\${arity}")
      .build();`,

    {
      label: "New Socket Snippet",
      type: "function",
    },
  ),

  // prop valueFrom
  snippetCompletion(
    `.setValueFrom(new ValueFromBuilder()
      .setKind("prop")
      .setPropPath(["\${path}"])
      .build())`,

    {
      label: "Value From Prop Snippet",
      type: "function",
    },
  ),

  // socket valueFrom
  snippetCompletion(
    `.setValueFrom(new ValueFromBuilder()
      .setKind("\${input:output}Socket")
      .setSocketName("\${name}")
      .build())`,

    {
      label: "Value From Socket Snippet",
      type: "function",
    },
  ),

  // add option to select box
  snippetCompletion(
    `.addOption("\${name}", "\${value}")`,

    {
      label: "Select Box Option Snippet",
      type: "function",
    },
  ),

  // aws region prop
  snippetCompletion(
    `const regionProp = new PropBuilder()
        .setKind("string")
        .setName("region")
        .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
        .setValueFrom(new ValueFromBuilder()
            .setKind("inputSocket")
            .setSocketName("Region")
            .build())
        .build();

    const regionSocket = new SocketDefinitionBuilder()
        .setName("Region")
        .setArity("one")
        .build();`,

    {
      label: "AWS Region Snippet",
      type: "function",
    },
  ),

  // aws tags prop
  snippetCompletion(
    `const tagsProp = new PropBuilder()
        .setKind("map")
        .setName("tags")
        .setWidget(new PropWidgetDefinitionBuilder().setKind("array").build())
        .addMapKeyFunc(new MapKeyFuncBuilder()
            .setKey("Name")
            .setValueFrom(new ValueFromBuilder()
              .setKind("prop")
              .setPropPath(["root", "si", "name"])
              .build())
        .build())
        .setEntry(new PropBuilder()
            .setKind("string")
            .setName("tag")
            .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
            .build())
        .build()`,

    {
      label: "AWS Tags Snippet",
      type: "function",
    },
  ),
];
