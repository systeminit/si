// package: si.external_api_gateway.aws.eks
// file: si-external-api-gateway/proto/si.external_api_gateway.aws.eks.proto

import * as jspb from "google-protobuf";
import * as si_external_api_gateway_proto_si_external_api_gateway_pb from "../../si-external-api-gateway/proto/si.external_api_gateway_pb";

export class CreateClusterRequest extends jspb.Message {
  hasContext(): boolean;
  clearContext(): void;
  getContext(): si_external_api_gateway_proto_si_external_api_gateway_pb.Context | undefined;
  setContext(value?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context): void;

  getClientRequestToken(): string;
  setClientRequestToken(value: string): void;

  hasLogging(): boolean;
  clearLogging(): void;
  getLogging(): Logging | undefined;
  setLogging(value?: Logging): void;

  getName(): string;
  setName(value: string): void;

  hasResourcesVpcConfig(): boolean;
  clearResourcesVpcConfig(): void;
  getResourcesVpcConfig(): CreateClusterRequest.VpcConfigRequest | undefined;
  setResourcesVpcConfig(value?: CreateClusterRequest.VpcConfigRequest): void;

  getRoleArn(): string;
  setRoleArn(value: string): void;

  clearTagsList(): void;
  getTagsList(): Array<Tag>;
  setTagsList(value: Array<Tag>): void;
  addTags(value?: Tag, index?: number): Tag;

  getVersion(): string;
  setVersion(value: string): void;

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
    clientRequestToken: string,
    logging?: Logging.AsObject,
    name: string,
    resourcesVpcConfig?: CreateClusterRequest.VpcConfigRequest.AsObject,
    roleArn: string,
    tagsList: Array<Tag.AsObject>,
    version: string,
  }

  export class VpcConfigRequest extends jspb.Message {
    getEndpointPrivateAccess(): boolean;
    setEndpointPrivateAccess(value: boolean): void;

    getEndpointPublicAccess(): boolean;
    setEndpointPublicAccess(value: boolean): void;

    clearPublicAccessCidrsList(): void;
    getPublicAccessCidrsList(): Array<string>;
    setPublicAccessCidrsList(value: Array<string>): void;
    addPublicAccessCidrs(value: string, index?: number): string;

    clearSecurityGroupIdsList(): void;
    getSecurityGroupIdsList(): Array<string>;
    setSecurityGroupIdsList(value: Array<string>): void;
    addSecurityGroupIds(value: string, index?: number): string;

    clearSubnetIdsList(): void;
    getSubnetIdsList(): Array<string>;
    setSubnetIdsList(value: Array<string>): void;
    addSubnetIds(value: string, index?: number): string;

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
      endpointPrivateAccess: boolean,
      endpointPublicAccess: boolean,
      publicAccessCidrsList: Array<string>,
      securityGroupIdsList: Array<string>,
      subnetIdsList: Array<string>,
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
    getArn(): string;
    setArn(value: string): void;

    hasCertificateAuthority(): boolean;
    clearCertificateAuthority(): void;
    getCertificateAuthority(): CreateClusterReply.Cluster.Certificate | undefined;
    setCertificateAuthority(value?: CreateClusterReply.Cluster.Certificate): void;

    getClientRequestToken(): string;
    setClientRequestToken(value: string): void;

    getCreatedAt(): string;
    setCreatedAt(value: string): void;

    getEndpoint(): string;
    setEndpoint(value: string): void;

    hasIdentity(): boolean;
    clearIdentity(): void;
    getIdentity(): CreateClusterReply.Cluster.Identity | undefined;
    setIdentity(value?: CreateClusterReply.Cluster.Identity): void;

    hasLogging(): boolean;
    clearLogging(): void;
    getLogging(): Logging | undefined;
    setLogging(value?: Logging): void;

    getName(): string;
    setName(value: string): void;

    getPlatformVersion(): string;
    setPlatformVersion(value: string): void;

    hasResourcesVpcConfig(): boolean;
    clearResourcesVpcConfig(): void;
    getResourcesVpcConfig(): CreateClusterReply.Cluster.VpcConfigResponse | undefined;
    setResourcesVpcConfig(value?: CreateClusterReply.Cluster.VpcConfigResponse): void;

    getRoleArn(): string;
    setRoleArn(value: string): void;

    getStatus(): string;
    setStatus(value: string): void;

    clearTagsList(): void;
    getTagsList(): Array<Tag>;
    setTagsList(value: Array<Tag>): void;
    addTags(value?: Tag, index?: number): Tag;

    getVersion(): string;
    setVersion(value: string): void;

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
      arn: string,
      certificateAuthority?: CreateClusterReply.Cluster.Certificate.AsObject,
      clientRequestToken: string,
      createdAt: string,
      endpoint: string,
      identity?: CreateClusterReply.Cluster.Identity.AsObject,
      logging?: Logging.AsObject,
      name: string,
      platformVersion: string,
      resourcesVpcConfig?: CreateClusterReply.Cluster.VpcConfigResponse.AsObject,
      roleArn: string,
      status: string,
      tagsList: Array<Tag.AsObject>,
      version: string,
    }

    export class Certificate extends jspb.Message {
      getData(): string;
      setData(value: string): void;

      serializeBinary(): Uint8Array;
      toObject(includeInstance?: boolean): Certificate.AsObject;
      static toObject(includeInstance: boolean, msg: Certificate): Certificate.AsObject;
      static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
      static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
      static serializeBinaryToWriter(message: Certificate, writer: jspb.BinaryWriter): void;
      static deserializeBinary(bytes: Uint8Array): Certificate;
      static deserializeBinaryFromReader(message: Certificate, reader: jspb.BinaryReader): Certificate;
    }

    export namespace Certificate {
      export type AsObject = {
        data: string,
      }
    }

    export class Identity extends jspb.Message {
      hasOidc(): boolean;
      clearOidc(): void;
      getOidc(): CreateClusterReply.Cluster.Identity.Oidc | undefined;
      setOidc(value?: CreateClusterReply.Cluster.Identity.Oidc): void;

      serializeBinary(): Uint8Array;
      toObject(includeInstance?: boolean): Identity.AsObject;
      static toObject(includeInstance: boolean, msg: Identity): Identity.AsObject;
      static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
      static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
      static serializeBinaryToWriter(message: Identity, writer: jspb.BinaryWriter): void;
      static deserializeBinary(bytes: Uint8Array): Identity;
      static deserializeBinaryFromReader(message: Identity, reader: jspb.BinaryReader): Identity;
    }

    export namespace Identity {
      export type AsObject = {
        oidc?: CreateClusterReply.Cluster.Identity.Oidc.AsObject,
      }

      export class Oidc extends jspb.Message {
        getIssuer(): string;
        setIssuer(value: string): void;

        serializeBinary(): Uint8Array;
        toObject(includeInstance?: boolean): Oidc.AsObject;
        static toObject(includeInstance: boolean, msg: Oidc): Oidc.AsObject;
        static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
        static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
        static serializeBinaryToWriter(message: Oidc, writer: jspb.BinaryWriter): void;
        static deserializeBinary(bytes: Uint8Array): Oidc;
        static deserializeBinaryFromReader(message: Oidc, reader: jspb.BinaryReader): Oidc;
      }

      export namespace Oidc {
        export type AsObject = {
          issuer: string,
        }
      }
    }

    export class VpcConfigResponse extends jspb.Message {
      getClusterSecurityGroupId(): string;
      setClusterSecurityGroupId(value: string): void;

      getEndpointPrivateAccess(): boolean;
      setEndpointPrivateAccess(value: boolean): void;

      getEndpointPublicAccess(): boolean;
      setEndpointPublicAccess(value: boolean): void;

      clearPublicAccessCidrsList(): void;
      getPublicAccessCidrsList(): Array<string>;
      setPublicAccessCidrsList(value: Array<string>): void;
      addPublicAccessCidrs(value: string, index?: number): string;

      clearSecurityGroupIdsList(): void;
      getSecurityGroupIdsList(): Array<string>;
      setSecurityGroupIdsList(value: Array<string>): void;
      addSecurityGroupIds(value: string, index?: number): string;

      clearSubnetIdsList(): void;
      getSubnetIdsList(): Array<string>;
      setSubnetIdsList(value: Array<string>): void;
      addSubnetIds(value: string, index?: number): string;

      getVpcId(): string;
      setVpcId(value: string): void;

      serializeBinary(): Uint8Array;
      toObject(includeInstance?: boolean): VpcConfigResponse.AsObject;
      static toObject(includeInstance: boolean, msg: VpcConfigResponse): VpcConfigResponse.AsObject;
      static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
      static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
      static serializeBinaryToWriter(message: VpcConfigResponse, writer: jspb.BinaryWriter): void;
      static deserializeBinary(bytes: Uint8Array): VpcConfigResponse;
      static deserializeBinaryFromReader(message: VpcConfigResponse, reader: jspb.BinaryReader): VpcConfigResponse;
    }

    export namespace VpcConfigResponse {
      export type AsObject = {
        clusterSecurityGroupId: string,
        endpointPrivateAccess: boolean,
        endpointPublicAccess: boolean,
        publicAccessCidrsList: Array<string>,
        securityGroupIdsList: Array<string>,
        subnetIdsList: Array<string>,
        vpcId: string,
      }
    }
  }
}

export class Logging extends jspb.Message {
  clearClusterLoggingList(): void;
  getClusterLoggingList(): Array<Logging.LogSetup>;
  setClusterLoggingList(value: Array<Logging.LogSetup>): void;
  addClusterLogging(value?: Logging.LogSetup, index?: number): Logging.LogSetup;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Logging.AsObject;
  static toObject(includeInstance: boolean, msg: Logging): Logging.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Logging, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Logging;
  static deserializeBinaryFromReader(message: Logging, reader: jspb.BinaryReader): Logging;
}

export namespace Logging {
  export type AsObject = {
    clusterLoggingList: Array<Logging.LogSetup.AsObject>,
  }

  export class LogSetup extends jspb.Message {
    getEnabled(): boolean;
    setEnabled(value: boolean): void;

    clearTypesList(): void;
    getTypesList(): Array<string>;
    setTypesList(value: Array<string>): void;
    addTypes(value: string, index?: number): string;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): LogSetup.AsObject;
    static toObject(includeInstance: boolean, msg: LogSetup): LogSetup.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: LogSetup, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): LogSetup;
    static deserializeBinaryFromReader(message: LogSetup, reader: jspb.BinaryReader): LogSetup;
  }

  export namespace LogSetup {
    export type AsObject = {
      enabled: boolean,
      typesList: Array<string>,
    }
  }
}

export class Tag extends jspb.Message {
  getKey(): string;
  setKey(value: string): void;

  getValue(): string;
  setValue(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Tag.AsObject;
  static toObject(includeInstance: boolean, msg: Tag): Tag.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Tag, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Tag;
  static deserializeBinaryFromReader(message: Tag, reader: jspb.BinaryReader): Tag;
}

export namespace Tag {
  export type AsObject = {
    key: string,
    value: string,
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

