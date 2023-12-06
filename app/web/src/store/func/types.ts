import { PropKind } from "@/api/sdf/dal/prop";
import { FuncArgument } from "@/api/sdf/dal/func";
import { ActionKind } from "@/store/fixes.store";

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

export interface ValidationAssociations {
  type: "validation";
  prototypes: ValidationPrototypeView[];
}

export interface ValidationPrototypeView {
  schemaVariantId: string;
  propId: string;
}

export interface AttributePrototypeArgumentView {
  funcArgumentId: string;
  id?: string;
  internalProviderId?: string;
}

export interface AttributePrototypeView {
  id: string;
  componentId?: string;
  propId?: string;
  externalProviderId?: string;
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
  | QualificationAssociations
  | ValidationAssociations;

export interface InputSourceSocket {
  schemaVariantId: string;
  internalProviderId: string;
  name: string;
}

export interface OutputSocket {
  schemaVariantId: string;
  externalProviderId: string;
  name: string;
}

export interface InputSourceProp {
  propId: string;
  kind: PropKind;
  schemaVariantId: string;
  internalProviderId?: string;
  path: string;
  name: string;
}

export interface OutputLocationProp {
  label: string;
  propId: string;
}

export interface OutputLocationOutputSocket {
  label: string;
  externalProviderId: string;
}

export type OutputLocation = OutputLocationProp | OutputLocationOutputSocket;

export interface CreateFuncAttributeOutputLocationProp {
  type: "prop";
  propId: string;
}

export interface CreateFuncAttributeOutputLocationOutputSocket {
  type: "outputSocket";
  externalProviderId: string;
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

export interface CreateFuncValidationOptions {
  type: "validationOptions";
  schemaVariantId: string;
  propToValidate: string;
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
  | CreateFuncQualificationOptions
  | CreateFuncValidationOptions;
