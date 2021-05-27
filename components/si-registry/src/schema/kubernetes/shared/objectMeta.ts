import {
  PropObject,
  RegistryEntry,
  ValidatorKind,
} from "../../../registryEntry";

export const objectMeta: RegistryEntry["properties"] = [
  {
    type: "string",
    name: "name",
    validation: [
      {
        kind: ValidatorKind.Regex,
        regex: "^[A-Za-z0-9](?:[A-Za-z0-9-]{0,251}[A-Za-z0-9])?$",
        message: "Kubernetes names must be valid DNS subdomains",
        link:
          "https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#dns-subdomain-names",
      },
      {
        kind: ValidatorKind.Required,
      },
    ],
  },
  {
    type: "string",
    name: "generateName",
  },
  {
    type: "string",
    name: "namespace",
  },
  {
    type: "map",
    name: "labels",
    valueProperty: {
      type: "string",
    },
  },
  {
    type: "map",
    name: "annotations",
    valueProperty: {
      type: "string",
    },
  },
];

export const objectMetaOptional: RegistryEntry["properties"] = [
  {
    type: "string",
    name: "name",
    validation: [
      {
        kind: ValidatorKind.Regex,
        regex: "^[A-Za-z0-9](?:[A-Za-z0-9-]{0,251}[A-Za-z0-9])?$",
        message: "Kubernetes names must be valid DNS subdomains",
        link:
          "https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#dns-subdomain-names",
      },
    ],
  },
  {
    type: "string",
    name: "generateName",
  },
  {
    type: "string",
    name: "namespace",
  },
  {
    type: "map",
    name: "labels",
    valueProperty: {
      type: "string",
    },
  },
  {
    type: "map",
    name: "annotations",
    valueProperty: {
      type: "string",
    },
  },
];

export const metadata: PropObject = {
  type: "object",
  name: "metadata",
  properties: objectMeta,
};

export const metadataOptional: PropObject = {
  type: "object",
  name: "metadata",
  properties: objectMetaOptional,
};
