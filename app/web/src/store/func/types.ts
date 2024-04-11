import { PropKind } from "@/api/sdf/dal/prop";
import { FuncArgument } from "@/api/sdf/dal/func";
import { ActionKind } from "@/store/actions.store";

export interface ActionAssociations {
  type: "action";
  schemaVariantIds: string[];
  kind?: ActionKind;
}

export type LeafInputLocation =
  | "code"
  | "deletedAt"
  | "domain"
  | "resource"
  | "secrets";

export interface AuthenticationAssociations {
  type: "authentication";
  schemaVariantIds: string[];
}

export interface CodeGenerationAssociations {
  type: "codeGeneration";
  schemaVariantIds: string[];
  componentIds: string[];
  inputs: LeafInputLocation[];
}

export interface QualificationAssociations {
  type: "qualification";
  schemaVariantIds: string[];
  componentIds: string[];
  inputs: LeafInputLocation[];
}

export interface AttributePrototypeArgumentView {
  funcArgumentId: string;
  id?: string;
  inputSocketId?: string;
}

export interface AttributePrototypeView {
  id: string;
  componentId?: string;
  schemaVariantId?: string;
  propId?: string;
  outputSocketId?: string;
  prototypeArguments: AttributePrototypeArgumentView[];
}

export interface AttributeAssociations {
  type: "attribute";
  prototypes: AttributePrototypeView[];
  arguments: FuncArgument[];
}

export type FuncAssociations =
  | AuthenticationAssociations
  | ActionAssociations
  | AttributeAssociations
  | CodeGenerationAssociations
  | QualificationAssociations;

export interface InputSourceSocket {
  schemaVariantId: string;
  inputSocketId: string;
  name: string;
}

export interface OutputSocket {
  schemaVariantId: string;
  outputSocketId: string;
  name: string;
}

export interface InputSourceProp {
  propId: string;
  kind: PropKind;
  schemaVariantId: string;
  inputSocketId?: string;
  path: string;
  name: string;
}

export interface OutputLocationProp {
  label: string;
  propId: string;
}

export interface OutputLocationOutputSocket {
  label: string;
  outputSocketId: string;
}

export type OutputLocation = OutputLocationProp | OutputLocationOutputSocket;

export interface CreateFuncAttributeOutputLocationProp {
  type: "prop";
  propId: string;
}

export interface CreateFuncAttributeOutputLocationOutputSocket {
  type: "outputSocket";
  outputSocketId: string;
}

export type CreateFuncOutputLocation =
  | CreateFuncAttributeOutputLocationOutputSocket
  | CreateFuncAttributeOutputLocationProp;

export interface CreateFuncAuthenticationOptions {
  type: "authenticationOptions";
  schemaVariantId: string;
}

export interface CreateFuncAttributeOptions {
  type: "attributeOptions";
  schemaVariantId: string;
  outputLocation: CreateFuncOutputLocation;
}

export interface CreateFuncActionOptions {
  type: "actionOptions";
  schemaVariantId: string;
  actionKind: ActionKind;
}

export interface CreateFuncQualificationOptions {
  type: "qualificationOptions";
  schemaVariantId: string;
}

export interface CreateFuncCodeGenerationOptions {
  type: "codeGenerationOptions";
  schemaVariantId: string;
}

export type CreateFuncOptions =
  | CreateFuncAuthenticationOptions
  | CreateFuncActionOptions
  | CreateFuncAttributeOptions
  | CreateFuncCodeGenerationOptions
  | CreateFuncQualificationOptions;
