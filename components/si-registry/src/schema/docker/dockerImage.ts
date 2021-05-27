import {
  RegistryEntry,
  ValidatorKind,
  SchematicKind,
  NodeKind,
  //Arity,
} from "../../registryEntry";

const dockerImage: RegistryEntry = {
  entityType: "dockerImage",
  nodeKind: NodeKind.Concrete,
  ui: {
    menu: [
      {
        name: "image",
        menuCategory: ["container", "docker"],
        schematicKind: SchematicKind.Component,
        rootEntityTypes: ["service"],
      },
    ],
  },
  inputs: [],
  properties: [
    {
      type: "string",
      name: "image",
      validation: [
        {
          kind: ValidatorKind.Regex,
          regex:
            "^(?:(?=[^:/]{1,253})(?!-)[a-zA-Z0-9-]{1,63}(?<!-)(?:.(?!-)[a-zA-Z0-9-]{1,63}(?<!-))*(?::[0-9]{1,5})?/)?((?![._-])(?:[a-z0-9._-]*)(?<![._-])(?:/(?![._-])[a-z0-9._-]*(?<![._-]))*)(?::(?![.-])[a-zA-Z0-9_.-]{1,128})?$",

          message: "invalid docker image string",
          link: "https://docs.docker.com/engine/reference/commandline/tag/",
        },
      ],
    },
    {
      type: "array",
      name: "ExposedPorts",
      itemProperty: {
        type: "string",
        validation: [
          {
            kind: ValidatorKind.Regex,
            regex: "\\d+\\/(tcp|udp)",
            message: "invalid exposed port entry; must be [numeric]/(tcp|udp)",
          },
        ],
      },
      validation: [
        {
          kind: ValidatorKind.Required,
          userDefined: true,
        },
      ],
    },
  ],
  qualifications: [
    {
      name: "dockerImageExistsInRegistry",
      title: "Docker image exists in registry",
      description:
        "The docker image and tag specified must be accessible via a docker pull.",
      link: "https://docs.docker.com/engine/reference/commandline/pull/",
    },
  ],
  actions: [
    {
      name: "deploy",
    },
    {
      name: "pull",
    },
  ],
  commands: [
    {
      name: "universal:deploy",
      description: "Deploy",
    },
  ],
};

export default dockerImage;
