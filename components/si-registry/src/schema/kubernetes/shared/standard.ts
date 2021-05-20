import {
  CodeKind,
  PropString,
  RegistryEntry,
  ValidatorKind,
} from "../../../registryEntry";
import _ from "lodash";

export function code(): RegistryEntry["code"] {
  return { kind: CodeKind.YAML };
}

export function apiVersion(apiVersion: string): PropString {
  return {
    type: "string",
    name: "apiVersion",
    defaultValue: apiVersion,
    validation: [
      {
        kind: ValidatorKind.Regex,
        regex: `^${apiVersion}$`,
        message: `API Version must be the one specified by kubernetes: ${apiVersion}`,
      },
    ],
  };
}

export function kind(kind: string): PropString {
  return {
    type: "string",
    name: "kind",
    defaultValue: _.capitalize(kind),
    validation: [
      {
        kind: ValidatorKind.Regex,
        regex: `^${kind}$`,
        message: `Kind must match the object type: ${kind}`,
      },
    ],
  };
}

export const qualifications = [
  {
    name: "kubeval",
    title: "kubeval check",
    description: "Kubeval linting",
    link: "https://www.kubeval.com/",
  },
];

export const actions = [{ name: "apply" }];

export const commands = [{ name: "apply", description: "kubectl apply" }];
