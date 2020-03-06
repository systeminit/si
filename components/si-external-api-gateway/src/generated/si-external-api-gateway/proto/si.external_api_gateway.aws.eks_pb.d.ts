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
  getResourcesVpcConfig(): VpcConfigRequest | undefined;
  setResourcesVpcConfig(value?: VpcConfigRequest): void;

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
    resourcesVpcConfig?: VpcConfigRequest.AsObject,
    roleArn: string,
    tagsList: Array<Tag.AsObject>,
    version: string,
  }
}

export class CreateClusterReply extends jspb.Message {
  hasCluster(): boolean;
  clearCluster(): void;
  getCluster(): Cluster | undefined;
  setCluster(value?: Cluster): void;

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
    cluster?: Cluster.AsObject,
    error?: Error.AsObject,
  }
}

export class CreateNodegroupRequest extends jspb.Message {
  hasContext(): boolean;
  clearContext(): void;
  getContext(): si_external_api_gateway_proto_si_external_api_gateway_pb.Context | undefined;
  setContext(value?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context): void;

  getClusterName(): string;
  setClusterName(value: string): void;

  getAmiType(): string;
  setAmiType(value: string): void;

  getClientRequestToken(): string;
  setClientRequestToken(value: string): void;

  getDiskSize(): number;
  setDiskSize(value: number): void;

  clearInstanceTypesList(): void;
  getInstanceTypesList(): Array<string>;
  setInstanceTypesList(value: Array<string>): void;
  addInstanceTypes(value: string, index?: number): string;

  clearLabelsList(): void;
  getLabelsList(): Array<Label>;
  setLabelsList(value: Array<Label>): void;
  addLabels(value?: Label, index?: number): Label;

  getNodegroupName(): string;
  setNodegroupName(value: string): void;

  getNodeRole(): string;
  setNodeRole(value: string): void;

  getReleaseVersion(): string;
  setReleaseVersion(value: string): void;

  hasRemoteAccess(): boolean;
  clearRemoteAccess(): void;
  getRemoteAccess(): RemoteAccessConfig | undefined;
  setRemoteAccess(value?: RemoteAccessConfig): void;

  hasScalingConfig(): boolean;
  clearScalingConfig(): void;
  getScalingConfig(): NodegroupScalingConfig | undefined;
  setScalingConfig(value?: NodegroupScalingConfig): void;

  clearSubnetsList(): void;
  getSubnetsList(): Array<string>;
  setSubnetsList(value: Array<string>): void;
  addSubnets(value: string, index?: number): string;

  clearTagsList(): void;
  getTagsList(): Array<Tag>;
  setTagsList(value: Array<Tag>): void;
  addTags(value?: Tag, index?: number): Tag;

  getVersion(): string;
  setVersion(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateNodegroupRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateNodegroupRequest): CreateNodegroupRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: CreateNodegroupRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateNodegroupRequest;
  static deserializeBinaryFromReader(message: CreateNodegroupRequest, reader: jspb.BinaryReader): CreateNodegroupRequest;
}

export namespace CreateNodegroupRequest {
  export type AsObject = {
    context?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context.AsObject,
    clusterName: string,
    amiType: string,
    clientRequestToken: string,
    diskSize: number,
    instanceTypesList: Array<string>,
    labelsList: Array<Label.AsObject>,
    nodegroupName: string,
    nodeRole: string,
    releaseVersion: string,
    remoteAccess?: RemoteAccessConfig.AsObject,
    scalingConfig?: NodegroupScalingConfig.AsObject,
    subnetsList: Array<string>,
    tagsList: Array<Tag.AsObject>,
    version: string,
  }
}

export class CreateNodegroupReply extends jspb.Message {
  hasNodegroup(): boolean;
  clearNodegroup(): void;
  getNodegroup(): Nodegroup | undefined;
  setNodegroup(value?: Nodegroup): void;

  hasError(): boolean;
  clearError(): void;
  getError(): Error | undefined;
  setError(value?: Error): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateNodegroupReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateNodegroupReply): CreateNodegroupReply.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: CreateNodegroupReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateNodegroupReply;
  static deserializeBinaryFromReader(message: CreateNodegroupReply, reader: jspb.BinaryReader): CreateNodegroupReply;
}

export namespace CreateNodegroupReply {
  export type AsObject = {
    nodegroup?: Nodegroup.AsObject,
    error?: Error.AsObject,
  }
}

export class DescribeClusterRequest extends jspb.Message {
  hasContext(): boolean;
  clearContext(): void;
  getContext(): si_external_api_gateway_proto_si_external_api_gateway_pb.Context | undefined;
  setContext(value?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context): void;

  getName(): string;
  setName(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DescribeClusterRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DescribeClusterRequest): DescribeClusterRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: DescribeClusterRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DescribeClusterRequest;
  static deserializeBinaryFromReader(message: DescribeClusterRequest, reader: jspb.BinaryReader): DescribeClusterRequest;
}

export namespace DescribeClusterRequest {
  export type AsObject = {
    context?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context.AsObject,
    name: string,
  }
}

export class DescribeClusterReply extends jspb.Message {
  hasCluster(): boolean;
  clearCluster(): void;
  getCluster(): Cluster | undefined;
  setCluster(value?: Cluster): void;

  hasError(): boolean;
  clearError(): void;
  getError(): Error | undefined;
  setError(value?: Error): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DescribeClusterReply.AsObject;
  static toObject(includeInstance: boolean, msg: DescribeClusterReply): DescribeClusterReply.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: DescribeClusterReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DescribeClusterReply;
  static deserializeBinaryFromReader(message: DescribeClusterReply, reader: jspb.BinaryReader): DescribeClusterReply;
}

export namespace DescribeClusterReply {
  export type AsObject = {
    cluster?: Cluster.AsObject,
    error?: Error.AsObject,
  }
}

export class DescribeNodegroupRequest extends jspb.Message {
  hasContext(): boolean;
  clearContext(): void;
  getContext(): si_external_api_gateway_proto_si_external_api_gateway_pb.Context | undefined;
  setContext(value?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context): void;

  getClusterName(): string;
  setClusterName(value: string): void;

  getNodegroupName(): string;
  setNodegroupName(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DescribeNodegroupRequest.AsObject;
  static toObject(includeInstance: boolean, msg: DescribeNodegroupRequest): DescribeNodegroupRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: DescribeNodegroupRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DescribeNodegroupRequest;
  static deserializeBinaryFromReader(message: DescribeNodegroupRequest, reader: jspb.BinaryReader): DescribeNodegroupRequest;
}

export namespace DescribeNodegroupRequest {
  export type AsObject = {
    context?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context.AsObject,
    clusterName: string,
    nodegroupName: string,
  }
}

export class DescribeNodegroupReply extends jspb.Message {
  hasNodegroup(): boolean;
  clearNodegroup(): void;
  getNodegroup(): Nodegroup | undefined;
  setNodegroup(value?: Nodegroup): void;

  hasError(): boolean;
  clearError(): void;
  getError(): Error | undefined;
  setError(value?: Error): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): DescribeNodegroupReply.AsObject;
  static toObject(includeInstance: boolean, msg: DescribeNodegroupReply): DescribeNodegroupReply.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: DescribeNodegroupReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): DescribeNodegroupReply;
  static deserializeBinaryFromReader(message: DescribeNodegroupReply, reader: jspb.BinaryReader): DescribeNodegroupReply;
}

export namespace DescribeNodegroupReply {
  export type AsObject = {
    nodegroup?: Nodegroup.AsObject,
    error?: Error.AsObject,
  }
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

export class Cluster extends jspb.Message {
  getArn(): string;
  setArn(value: string): void;

  hasCertificateAuthority(): boolean;
  clearCertificateAuthority(): void;
  getCertificateAuthority(): Certificate | undefined;
  setCertificateAuthority(value?: Certificate): void;

  getClientRequestToken(): string;
  setClientRequestToken(value: string): void;

  getCreatedAt(): string;
  setCreatedAt(value: string): void;

  getEndpoint(): string;
  setEndpoint(value: string): void;

  hasIdentity(): boolean;
  clearIdentity(): void;
  getIdentity(): Cluster.Identity | undefined;
  setIdentity(value?: Cluster.Identity): void;

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
  getResourcesVpcConfig(): VpcConfigResponse | undefined;
  setResourcesVpcConfig(value?: VpcConfigResponse): void;

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
    certificateAuthority?: Certificate.AsObject,
    clientRequestToken: string,
    createdAt: string,
    endpoint: string,
    identity?: Cluster.Identity.AsObject,
    logging?: Logging.AsObject,
    name: string,
    platformVersion: string,
    resourcesVpcConfig?: VpcConfigResponse.AsObject,
    roleArn: string,
    status: string,
    tagsList: Array<Tag.AsObject>,
    version: string,
  }

  export class Identity extends jspb.Message {
    hasOidc(): boolean;
    clearOidc(): void;
    getOidc(): Cluster.Identity.Oidc | undefined;
    setOidc(value?: Cluster.Identity.Oidc): void;

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
      oidc?: Cluster.Identity.Oidc.AsObject,
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
}

export class Label extends jspb.Message {
  getKey(): string;
  setKey(value: string): void;

  getValue(): string;
  setValue(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Label.AsObject;
  static toObject(includeInstance: boolean, msg: Label): Label.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Label, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Label;
  static deserializeBinaryFromReader(message: Label, reader: jspb.BinaryReader): Label;
}

export namespace Label {
  export type AsObject = {
    key: string,
    value: string,
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

export class Nodegroup extends jspb.Message {
  getAmiType(): string;
  setAmiType(value: string): void;

  getClusterName(): string;
  setClusterName(value: string): void;

  getCreatedAt(): string;
  setCreatedAt(value: string): void;

  getDiskSize(): number;
  setDiskSize(value: number): void;

  hasHealth(): boolean;
  clearHealth(): void;
  getHealth(): NodegroupHealth | undefined;
  setHealth(value?: NodegroupHealth): void;

  clearInstanceTypesList(): void;
  getInstanceTypesList(): Array<string>;
  setInstanceTypesList(value: Array<string>): void;
  addInstanceTypes(value: string, index?: number): string;

  clearLabelsList(): void;
  getLabelsList(): Array<Label>;
  setLabelsList(value: Array<Label>): void;
  addLabels(value?: Label, index?: number): Label;

  getModifiedAt(): string;
  setModifiedAt(value: string): void;

  getNodegroupArn(): string;
  setNodegroupArn(value: string): void;

  getNodegroupName(): string;
  setNodegroupName(value: string): void;

  getNodeRole(): string;
  setNodeRole(value: string): void;

  getReleaseVersion(): string;
  setReleaseVersion(value: string): void;

  hasRemoteAccess(): boolean;
  clearRemoteAccess(): void;
  getRemoteAccess(): RemoteAccessConfig | undefined;
  setRemoteAccess(value?: RemoteAccessConfig): void;

  hasResources(): boolean;
  clearResources(): void;
  getResources(): NodegroupResources | undefined;
  setResources(value?: NodegroupResources): void;

  hasScalingConfig(): boolean;
  clearScalingConfig(): void;
  getScalingConfig(): NodegroupScalingConfig | undefined;
  setScalingConfig(value?: NodegroupScalingConfig): void;

  getStatus(): string;
  setStatus(value: string): void;

  clearSubnetsList(): void;
  getSubnetsList(): Array<string>;
  setSubnetsList(value: Array<string>): void;
  addSubnets(value: string, index?: number): string;

  clearTagsList(): void;
  getTagsList(): Array<Tag>;
  setTagsList(value: Array<Tag>): void;
  addTags(value?: Tag, index?: number): Tag;

  getVersion(): string;
  setVersion(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Nodegroup.AsObject;
  static toObject(includeInstance: boolean, msg: Nodegroup): Nodegroup.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Nodegroup, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Nodegroup;
  static deserializeBinaryFromReader(message: Nodegroup, reader: jspb.BinaryReader): Nodegroup;
}

export namespace Nodegroup {
  export type AsObject = {
    amiType: string,
    clusterName: string,
    createdAt: string,
    diskSize: number,
    health?: NodegroupHealth.AsObject,
    instanceTypesList: Array<string>,
    labelsList: Array<Label.AsObject>,
    modifiedAt: string,
    nodegroupArn: string,
    nodegroupName: string,
    nodeRole: string,
    releaseVersion: string,
    remoteAccess?: RemoteAccessConfig.AsObject,
    resources?: NodegroupResources.AsObject,
    scalingConfig?: NodegroupScalingConfig.AsObject,
    status: string,
    subnetsList: Array<string>,
    tagsList: Array<Tag.AsObject>,
    version: string,
  }
}

export class NodegroupHealth extends jspb.Message {
  clearIssuesList(): void;
  getIssuesList(): Array<NodegroupHealth.Issue>;
  setIssuesList(value: Array<NodegroupHealth.Issue>): void;
  addIssues(value?: NodegroupHealth.Issue, index?: number): NodegroupHealth.Issue;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): NodegroupHealth.AsObject;
  static toObject(includeInstance: boolean, msg: NodegroupHealth): NodegroupHealth.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: NodegroupHealth, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): NodegroupHealth;
  static deserializeBinaryFromReader(message: NodegroupHealth, reader: jspb.BinaryReader): NodegroupHealth;
}

export namespace NodegroupHealth {
  export type AsObject = {
    issuesList: Array<NodegroupHealth.Issue.AsObject>,
  }

  export class Issue extends jspb.Message {
    getCode(): string;
    setCode(value: string): void;

    getMessage(): string;
    setMessage(value: string): void;

    clearResourceIdsList(): void;
    getResourceIdsList(): Array<string>;
    setResourceIdsList(value: Array<string>): void;
    addResourceIds(value: string, index?: number): string;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): Issue.AsObject;
    static toObject(includeInstance: boolean, msg: Issue): Issue.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: Issue, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): Issue;
    static deserializeBinaryFromReader(message: Issue, reader: jspb.BinaryReader): Issue;
  }

  export namespace Issue {
    export type AsObject = {
      code: string,
      message: string,
      resourceIdsList: Array<string>,
    }
  }
}

export class NodegroupResources extends jspb.Message {
  clearAutoScalingGroupsList(): void;
  getAutoScalingGroupsList(): Array<NodegroupResources.AutoScalingGroup>;
  setAutoScalingGroupsList(value: Array<NodegroupResources.AutoScalingGroup>): void;
  addAutoScalingGroups(value?: NodegroupResources.AutoScalingGroup, index?: number): NodegroupResources.AutoScalingGroup;

  getRemoteAccessSecurityGroup(): string;
  setRemoteAccessSecurityGroup(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): NodegroupResources.AsObject;
  static toObject(includeInstance: boolean, msg: NodegroupResources): NodegroupResources.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: NodegroupResources, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): NodegroupResources;
  static deserializeBinaryFromReader(message: NodegroupResources, reader: jspb.BinaryReader): NodegroupResources;
}

export namespace NodegroupResources {
  export type AsObject = {
    autoScalingGroupsList: Array<NodegroupResources.AutoScalingGroup.AsObject>,
    remoteAccessSecurityGroup: string,
  }

  export class AutoScalingGroup extends jspb.Message {
    getName(): string;
    setName(value: string): void;

    serializeBinary(): Uint8Array;
    toObject(includeInstance?: boolean): AutoScalingGroup.AsObject;
    static toObject(includeInstance: boolean, msg: AutoScalingGroup): AutoScalingGroup.AsObject;
    static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
    static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
    static serializeBinaryToWriter(message: AutoScalingGroup, writer: jspb.BinaryWriter): void;
    static deserializeBinary(bytes: Uint8Array): AutoScalingGroup;
    static deserializeBinaryFromReader(message: AutoScalingGroup, reader: jspb.BinaryReader): AutoScalingGroup;
  }

  export namespace AutoScalingGroup {
    export type AsObject = {
      name: string,
    }
  }
}

export class NodegroupScalingConfig extends jspb.Message {
  getDesiredSize(): number;
  setDesiredSize(value: number): void;

  getMaxSize(): number;
  setMaxSize(value: number): void;

  getMinSize(): number;
  setMinSize(value: number): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): NodegroupScalingConfig.AsObject;
  static toObject(includeInstance: boolean, msg: NodegroupScalingConfig): NodegroupScalingConfig.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: NodegroupScalingConfig, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): NodegroupScalingConfig;
  static deserializeBinaryFromReader(message: NodegroupScalingConfig, reader: jspb.BinaryReader): NodegroupScalingConfig;
}

export namespace NodegroupScalingConfig {
  export type AsObject = {
    desiredSize: number,
    maxSize: number,
    minSize: number,
  }
}

export class RemoteAccessConfig extends jspb.Message {
  getEc2SshKey(): string;
  setEc2SshKey(value: string): void;

  clearSourcesecuritygroupsList(): void;
  getSourcesecuritygroupsList(): Array<string>;
  setSourcesecuritygroupsList(value: Array<string>): void;
  addSourcesecuritygroups(value: string, index?: number): string;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): RemoteAccessConfig.AsObject;
  static toObject(includeInstance: boolean, msg: RemoteAccessConfig): RemoteAccessConfig.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: RemoteAccessConfig, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): RemoteAccessConfig;
  static deserializeBinaryFromReader(message: RemoteAccessConfig, reader: jspb.BinaryReader): RemoteAccessConfig;
}

export namespace RemoteAccessConfig {
  export type AsObject = {
    ec2SshKey: string,
    sourcesecuritygroupsList: Array<string>,
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

