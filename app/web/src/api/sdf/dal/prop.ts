export type PropId = string;

export enum PropKind {
  Array = "array",
  Boolean = "boolean",
  Float = "float",
  Integer = "integer",
  Json = "json",
  Object = "object",
  String = "string",
  Map = "map",
}

export interface Prop {
  id: PropId;
  kind: PropKind;
  name: string;
  path: string;
  // this is for output sources
  eligibleToReceiveData: boolean;
  // this is for input sources
  eligibleToSendData: boolean;
  hidden: boolean;
}

/**
 * Valid top-level props of a component.
 */
export type TopLevelProp =
  | "code"
  | "domain"
  | "resource_value"
  | "resource"
  | "qualification"
  | "secret_definition"
  | "secrets"
  | "si";
