// package: si.external_api_gateway
// file: si-external-api-gateway/proto/si.external_api_gateway.proto

import * as jspb from "google-protobuf";

export class Context extends jspb.Message {
  getUserId(): string;
  setUserId(value: string): void;

  getBillingAccountId(): string;
  setBillingAccountId(value: string): void;

  getOrganizationId(): string;
  setOrganizationId(value: string): void;

  getWorkspaceId(): string;
  setWorkspaceId(value: string): void;

  getIntegrationInstanceId(): string;
  setIntegrationInstanceId(value: string): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): Context.AsObject;
  static toObject(includeInstance: boolean, msg: Context): Context.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: Context, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): Context;
  static deserializeBinaryFromReader(message: Context, reader: jspb.BinaryReader): Context;
}

export namespace Context {
  export type AsObject = {
    userId: string,
    billingAccountId: string,
    organizationId: string,
    workspaceId: string,
    integrationInstanceId: string,
  }
}

