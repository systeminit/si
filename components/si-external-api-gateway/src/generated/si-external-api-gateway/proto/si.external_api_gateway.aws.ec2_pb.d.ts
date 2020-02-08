// package: si.external_api_gateway.aws.ec2
// file: si-external-api-gateway/proto/si.external_api_gateway.aws.ec2.proto

import * as jspb from "google-protobuf";
import * as si_external_api_gateway_proto_si_external_api_gateway_pb from "../../si-external-api-gateway/proto/si.external_api_gateway_pb";

export class CreateKeyPairRequest extends jspb.Message {
  hasContext(): boolean;
  clearContext(): void;
  getContext(): si_external_api_gateway_proto_si_external_api_gateway_pb.Context | undefined;
  setContext(value?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context): void;

  getKeyname(): string;
  setKeyname(value: string): void;

  getDryRun(): boolean;
  setDryRun(value: boolean): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateKeyPairRequest.AsObject;
  static toObject(includeInstance: boolean, msg: CreateKeyPairRequest): CreateKeyPairRequest.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: CreateKeyPairRequest, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateKeyPairRequest;
  static deserializeBinaryFromReader(message: CreateKeyPairRequest, reader: jspb.BinaryReader): CreateKeyPairRequest;
}

export namespace CreateKeyPairRequest {
  export type AsObject = {
    context?: si_external_api_gateway_proto_si_external_api_gateway_pb.Context.AsObject,
    keyname: string,
    dryRun: boolean,
  }
}

export class CreateKeyPairReply extends jspb.Message {
  getRequestId(): string;
  setRequestId(value: string): void;

  getKeyFingerprint(): string;
  setKeyFingerprint(value: string): void;

  getKeyMaterial(): string;
  setKeyMaterial(value: string): void;

  getKeyName(): string;
  setKeyName(value: string): void;

  getKeyPairId(): string;
  setKeyPairId(value: string): void;

  hasError(): boolean;
  clearError(): void;
  getError(): Error | undefined;
  setError(value?: Error): void;

  serializeBinary(): Uint8Array;
  toObject(includeInstance?: boolean): CreateKeyPairReply.AsObject;
  static toObject(includeInstance: boolean, msg: CreateKeyPairReply): CreateKeyPairReply.AsObject;
  static extensions: {[key: number]: jspb.ExtensionFieldInfo<jspb.Message>};
  static extensionsBinary: {[key: number]: jspb.ExtensionFieldBinaryInfo<jspb.Message>};
  static serializeBinaryToWriter(message: CreateKeyPairReply, writer: jspb.BinaryWriter): void;
  static deserializeBinary(bytes: Uint8Array): CreateKeyPairReply;
  static deserializeBinaryFromReader(message: CreateKeyPairReply, reader: jspb.BinaryReader): CreateKeyPairReply;
}

export namespace CreateKeyPairReply {
  export type AsObject = {
    requestId: string,
    keyFingerprint: string,
    keyMaterial: string,
    keyName: string,
    keyPairId: string,
    error?: Error.AsObject,
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

