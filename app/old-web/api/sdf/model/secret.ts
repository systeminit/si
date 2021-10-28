import { ISiStorable } from "@/api/sdf/model/siStorable";

export enum SecretObjectType {
  Credential = "credential",
}

export namespace SecretObjectType {
  export function labelFor(secretObjectType: SecretObjectType): string {
    switch (secretObjectType) {
      case SecretObjectType.Credential:
        return "Credential";
      default:
        throw Error(`Unknown SecretObjectType variant: ${secretObjectType}`);
    }
  }
}

export enum SecretKind {
  DockerHub = "dockerHub",
  AwsAccessKey = "awsAccessKey",
  HelmRepo = "helmRepo",
  AzureServicePrincipal = "azureServicePrincipal",
}

export namespace SecretKind {
  export function labelFor(secretKind: SecretKind): string {
    switch (secretKind) {
      case SecretKind.AwsAccessKey:
        return "AWS Access key";
      case SecretKind.DockerHub:
        return "Docker Hub";
      case SecretKind.HelmRepo:
        return "Helm Repository";
      case SecretKind.AzureServicePrincipal:
        return "Azure Service Principal";
      default:
        throw Error(`Unknown SecretKind variant: ${secretKind}`);
    }
  }

  export function objectTypeFor(secretKind: SecretKind): SecretObjectType {
    switch (secretKind) {
      case SecretKind.AwsAccessKey:
      case SecretKind.DockerHub:
      case SecretKind.HelmRepo:
      case SecretKind.AzureServicePrincipal:
        return SecretObjectType.Credential;
      default:
        throw Error(`Unknown SecretKind variant: ${secretKind}`);
    }
  }

  export function isACredential(secretKind: SecretKind): boolean {
    return objectTypeFor(secretKind) == SecretObjectType.Credential;
  }

  export function selectPropOptionFor(
    secretKind: SecretKind,
  ): { label: string; value: string } {
    return {
      label: labelFor(secretKind),
      value: secretKind,
    };
  }

  export function selectPropOptions(): { label: string; value: string }[] {
    return [
      selectPropOptionFor(SecretKind.AwsAccessKey),
      selectPropOptionFor(SecretKind.DockerHub),
      selectPropOptionFor(SecretKind.HelmRepo),
      selectPropOptionFor(SecretKind.AzureServicePrincipal),
    ];
  }
}

export enum SecretVersion {
  V1 = "v1",
}

export namespace SecretVersion {
  export function defaultValue(): SecretVersion {
    return SecretVersion.V1;
  }
}

export enum SecretAlgorithm {
  Sealedbox = "sealedbox",
}

export namespace SecretAlgorithm {
  export function defaultValue(): SecretAlgorithm {
    return SecretAlgorithm.Sealedbox;
  }
}

export interface ISecret {
  id: string;
  name: string;
  objectType: SecretObjectType;
  kind: SecretKind;
  siStorable: ISiStorable;
}

export class Secret implements ISecret {
  id: ISecret["id"];
  name: ISecret["name"];
  objectType: ISecret["objectType"];
  kind: ISecret["kind"];
  siStorable: ISecret["siStorable"];

  constructor(args: ISecret) {
    this.id = args.id;
    this.name = args.name;
    this.objectType = args.objectType;
    this.kind = args.kind;
    this.siStorable = args.siStorable;
  }
}
