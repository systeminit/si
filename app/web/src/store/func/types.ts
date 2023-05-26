import { PropKind } from "@/api/sdf/dal/prop";
import { FuncArgument } from "@/api/sdf/dal/func";
import { ActionKind } from "@/store/fixes.store";

export interface ActionAssociations {
  type: "action";
  schemaVariantIds: string[];
  kind: ActionKind;
}

export type LeafInputLocation = "code" | "deletedAt" | "domain" | "resource";

export interface CodeGenerationAssociations {
  type: "codeGeneration";
  schemaVariantIds: string[];
  componentIds: string[];
  inputs: LeafInputLocation[];
}

export interface ConfirmationFuncDescriptionContents {
  name: string;
  success_description?: string;
  failure_description?: string;
  provider?: string;
}

export interface FuncDescriptionContents {
  Confirmation: ConfirmationFuncDescriptionContents;
}

export interface FuncDescriptionView {
  schemaVariantId: string;
  contents: FuncDescriptionContents;
}

export interface ConfirmationAssociations {
  type: "confirmation";
  schemaVariantIds: string[];
  componentIds: string[];
  descriptions: FuncDescriptionView[];
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
  id: string;
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
  | ActionAssociations
  | AttributeAssociations
  | CodeGenerationAssociations
  | ConfirmationAssociations
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

export interface CreateFuncAttributeOptions {
  type: "attributeOptions";
  valueId: string;
  parentValueId?: string;
  componentId: string;
  schemaVariantId: string;
  schemaId: string;
  currentFuncId: string;
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
