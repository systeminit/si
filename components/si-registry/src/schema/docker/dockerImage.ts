import {
  RegistryEntry,
  MenuCategory,
  ValidatorKind,
  SchematicKind,
  Arity,
} from "../../registryEntry";

const dockerImage: RegistryEntry = {
  entityType: "dockerImage",
  ui: {
    menuCategory: MenuCategory.Docker,
    menuDisplayName: "docker image",
    schematicKinds: [SchematicKind.Component],
  },
  inputs: [
    {
      name: "image",
      types: ["service"],
      edgeKind: "configures",
      arity: Arity.One,
    },
  ],
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
  ],
  qualifications: [
    {
      name: "dockerImageExistsInRegistry",
      title: "Docker image exists in registry",
      description:
        "The docker image and tag specified must be accessible via a docker pull.",
      link: "https://docs.docker.com/engine/reference/commandline/pull/",
    },
    {
      name: "dockerImageIsTrue",
      title: "Just here for a minute",
      description:
        "The docker image and tag specified must be accessible via a docker pull.",
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
