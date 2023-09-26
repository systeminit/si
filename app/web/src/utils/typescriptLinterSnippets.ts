import { snippetCompletion } from "@codemirror/autocomplete";

export const snippets = [
  snippetCompletion(
    `const \${propName} = new PropBuilder()
      .setName("\${name}")
      .setKind("string")
      .setWidget(new PropWidgetDefinitionBuilder().setKind("text")
      .build())
    .build();`,
    {
      label: "Textbox Snippet",
      type: "function",
    },
  ),

  snippetCompletion(
    `const \${socketName} = new SocketDefinitionBuilder()
      .setName("\${name}")
      .setArity("\${arity}")
      .build();`,

    {
      label: "Socket Snippet",
      type: "function",
    },
  ),
];
