import { StandardModel } from "@/api/sdf/dal/standard_model";

export interface Secret extends StandardModel {
  name: string;
  objectType: SecretObjectType;
  kind: SecretKind;
}

export enum SecretObjectType {
  Credential = "credential",
}

export enum SecretKind {
  DockerHub = "dockerHub",
  AwsAccessKey = "awsAccessKey",
  HelmRepo = "helmRepo",
  AzureServicePrincipal = "azureServicePrincipal",
}

export enum SecretVersion {
  V1 = "v1",
}

export enum SecretAlgorithm {
  Sealedbox = "sealedbox",
}

export interface SecretKindFields {
  secretKind: SecretKind;
  displayName: string;
  fields: { keyName: string; displayName: string; password: boolean }[];
}
