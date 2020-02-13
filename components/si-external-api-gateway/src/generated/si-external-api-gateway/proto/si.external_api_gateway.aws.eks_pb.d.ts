// package: si.external_api_gateway.aws.eks
// file: si-external-api-gateway/proto/si.external_api_gateway.aws.eks.proto

import * as jspb from "google-protobuf";
import * as si_external_api_gateway_proto_si_external_api_gateway_pb from "../../si-external-api-gateway/proto/si.external_api_gateway_pb";

export class CreateClusterRequest extends jspb.Message {
  hasContext(): boolean;
  clearContext(): void;
  getContext(): si_external_api_gateway_proto_si_external_api_gateway_pb.Context | undefined;
  setContext(value?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context): void;

  getName(): string;
  setName(value: string): void;

  getVersion(): string;
  setVersion(value: string): void;

  getRoleArn(): string;
  setRoleArn(value: string): void;

  hasLogging(): boolean;
  clearLogging(): void;
  getLogging(): CreateClusterRequest.ClusterLogging | undefined;
  setLogging(value?: CreateClusterRequest.ClusterLogging): void;

  getClientRequestToken(): string;
  setClientRequestToken(value: string): void;

  hasResourcesVpcConfig(): boolean;
  clearResourcesVpcConfig(): void;
  getResourcesVpcConfig(): CreateClusterRequest.VpcConfigRequest | undefined;
  setResourcesVpcConfig(value?: CreateClusterRequest.VpcConfigRequest): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateClusterRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateClusterRequest): CreateClusterRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: CreateClusterRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateClusterRequest;
  static deserializeBinaryFromReader(message: CreateClusterRequest, reader: jspb.BinaryReader): CreateClusterRequest;
}

export namespace CreateClusterRequest {
  export type AsObject = {
    context?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context.AsObject,
    name: string,
    version: string,
    roleArn: string,
    logging?: CreateClusterRequest.ClusterLogging.AsObject,
    clientRequestToken: string,
    resourcesVpcConfig?: CreateClusterRequest.VpcConfigRequest.AsObject,
  }

  export class ClusterLogging extends jspb.Message {
    clearTypesList(): void;
    getTypesList(): Array<string>;
    setTypesList(value: Array<string>): void;
    addTypes(value: string, index?: number): string;

    getEnabled(): boolean;
    setEnabled(value: boolean): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): ClusterLogging.AsObject;
    static toObject(includeInstance: boolean, msg: ClusterLogging): ClusterLogging.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: ClusterLogging, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): ClusterLogging;
    static deserializeBinaryFromReader(message: ClusterLogging, reader: jspb.BinaryReader): ClusterLogging;
  }

  export namespace ClusterLogging {
    export type AsObject = {
      typesList: Array<string>,
      enabled: boolean,
    }
  }

  export class VpcConfigRequest extends jspb.Message {
    clearSubnetIdsList(): void;
    getSubnetIdsList(): Array<string>;
    setSubnetIdsList(value: Array<string>): void;
    addSubnetIds(value: string, index?: number): string;

    clearSecurityGroupIdsList(): void;
    getSecurityGroupIdsList(): Array<string>;
    setSecurityGroupIdsList(value: Array<string>): void;
    addSecurityGroupIds(value: string, index?: number): string;

    getEndpointPublicAccess(): boolean;
    setEndpointPublicAccess(value: boolean): void;

    getEndpointPrivateAccess(): boolean;
    setEndpointPrivateAccess(value: boolean): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): VpcConfigRequest.AsObject;
    static toObject(includeInstance: boolean, msg: VpcConfigRequest): VpcConfigRequest.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: VpcConfigRequest, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): VpcConfigRequest;
    static deserializeBinaryFromReader(message: VpcConfigRequest, reader: jspb.BinaryReader): VpcConfigRequest;
  }

  export namespace VpcConfigRequest {
    export type AsObject = {
      subnetIdsList: Array<string>,
      securityGroupIdsList: Array<string>,
      endpointPublicAccess: boolean,
      endpointPrivateAccess: boolean,
    }
  }
}

export class CreateClusterReply extends jspb.Message {
  hasCluster(): boolean;
  clearCluster(): void;
  getCluster(): CreateClusterReply.Cluster | undefined;
  setCluster(value?: CreateClusterReply.Cluster): void;

  hasError(): boolean;
  clearError(): void;
  getError(): Error | undefined;
  setError(value?: Error): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateClusterReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateClusterReply): CreateClusterReply.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: CreateClusterReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateClusterReply;
  static deserializeBinaryFromReader(message: CreateClusterReply, reader: jspb.BinaryReader): CreateClusterReply;
}

export namespace CreateClusterReply {
  export type AsObject = {
    cluster?: CreateClusterReply.Cluster.AsObject,
    error?: Error.AsObject,
  }

  export class Cluster extends jspb.Message {
    getCreatedAt(): string;
    setCreatedAt(value: string): void;

    hasResourcesVpcConfig(): boolean;
    clearResourcesVpcConfig(): void;
    getResourcesVpcConfig(): CreateClusterReply.Cluster.ResourcesVpcConfig | undefined;
    setResourcesVpcConfig(value?: CreateClusterReply.Cluster.ResourcesVpcConfig): void;

    hasLogging(): boolean;
    clearLogging(): void;
    getLogging(): CreateClusterReply.Cluster.ClusterLogging | undefined;
    setLogging(value?: CreateClusterReply.Cluster.ClusterLogging): void;

    hasCertificateAuthority(): boolean;
    clearCertificateAuthority(): void;
    getCertificateAuthority(): CreateClusterReply.Cluster.CertificateAuthority | undefined;
    setCertificateAuthority(value?: CreateClusterReply.Cluster.CertificateAuthority): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Cluster.AsObject;
    static toObject(includeInstance: boolean, msg: Cluster): Cluster.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Cluster, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Cluster;
    static deserializeBinaryFromReader(message: Cluster, reader: jspb.BinaryReader): Cluster;
  }

  export namespace Cluster {
    export type AsObject = {
      createdAt: string,
      resourcesVpcConfig?: CreateClusterReply.Cluster.ResourcesVpcConfig.AsObject,
      logging?: CreateClusterReply.Cluster.ClusterLogging.AsObject,
      certificateAuthority?: CreateClusterReply.Cluster.CertificateAuthority.AsObject,
    }

    export class ResourcesVpcConfig extends jspb.Message {
      clearSubnetIdsList(): void;
      getSubnetIdsList(): Array<string>;
      setSubnetIdsList(value: Array<string>): void;
      addSubnetIds(value: string, index?: number): string;

      clearSecurityGroupIdsList(): void;
      getSecurityGroupIdsList(): Array<string>;
      setSecurityGroupIdsList(value: Array<string>): void;
      addSecurityGroupIds(value: string, index?: number): string;

      serializeBinary(): Uint8Array;
      toObject(includeInstance?: boolean): ResourcesVpcConfig.AsObject;
      static toObject(includeInstance: boolean, msg: ResourcesVpcConfig): ResourcesVpcConfig.AsObject;
      static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
      static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
      static serializeBinaryToWriter(message: ResourcesVpcConfig, writer: jspb.BinaryWriter): void;
      static deserializeBinary(bytes: Uint8Array): ResourcesVpcConfig;
      static deserializeBinaryFromReader(message: ResourcesVpcConfig, reader: jspb.BinaryReader): ResourcesVpcConfig;
    }

    export namespace ResourcesVpcConfig {
      export type AsObject = {
        subnetIdsList: Array<string>,
        securityGroupIdsList: Array<string>,
      }
    }

    export class ClusterLogging extends jspb.Message {
      clearTypesList(): void;
      getTypesList(): Array<string>;
      setTypesList(value: Array<string>): void;
      addTypes(value: string, index?: number): string;

      serializeBinary(): Uint8Array;
      toObject(includeInstance?: boolean): ClusterLogging.AsObject;
      static toObject(includeInstance: boolean, msg: ClusterLogging): ClusterLogging.AsObject;
      static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
      static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
      static serializeBinaryToWriter(message: ClusterLogging, writer: jspb.BinaryWriter): void;
      static deserializeBinary(bytes: Uint8Array): ClusterLogging;
      static deserializeBinaryFromReader(message: ClusterLogging, reader: jspb.BinaryReader): ClusterLogging;
    }

    export namespace ClusterLogging {
      export type AsObject = {
        typesList: Array<string>,
      }
    }

    export class CertificateAuthority extends jspb.Message {
      getData(): string;
      setData(value: string): void;

      serializeBinary(): Uint8Array;
      toObject(includeInstance?: boolean): CertificateAuthority.AsObject;
      static toObject(includeInstance: boolean, msg: CertificateAuthority): CertificateAuthority.AsObject;
      static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
      static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
      static serializeBinaryToWriter(message: CertificateAuthority, writer: jspb.BinaryWriter): void;
      static deserializeBinary(bytes: Uint8Array): CertificateAuthority;
      static deserializeBinaryFromReader(message: CertificateAuthority, reader: jspb.BinaryReader): CertificateAuthority;
    }

    export namespace CertificateAuthority {
      export type AsObject = {
        data: string,
      }
    }
  }
}

export class Error extends jspb.Message {
  getCode(): string;
  setCode(value: string): void;

  getMessage(): string;
  setMessage(value: string): void;

  getRequestId(): string;
  setRequestId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Error.AsObject;
  static toObject(includeInstance: boolean, msg: Error): Error.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Error, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Error;
  static deserializeBinaryFromReader(message: Error, reader: jspb.BinaryReader): Error;
}

export namespace Error {
  export type AsObject = {
    code: string,
    message: string,
    requestId: string,
  }
}

export class Filter extends jspb.Message {
  getName(): string;
  setName(value: string): void;

  clearValuesList(): void;
  getValuesList(): Array<string>;
  setValuesList(value: Array<string>): void;
  addValues(value: string, index?: number): string;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Filter.AsObject;
  static toObject(includeInstance: boolean, msg: Filter): Filter.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Filter, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Filter;
  static deserializeBinaryFromReader(message: Filter, reader: jspb.BinaryReader): Filter;
}

export namespace Filter {
  export type AsObject = {
    name: string,
    valuesList: Array<string>,
  }
}

