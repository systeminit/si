import { PropKind } from "@/api/sdf/dal/prop";
import { FuncArgument } from "@/api/sdf/dal/func";

export interface CodeGenerationAssociations {
  type: "codeGeneration";
  schemaVariantIds: string[];
  componentIds: string[];
}

export interface ConfirmationAssociations {
  type: "confirmation";
  schemaVariantIds: string[];
  componentIds: string[];
}

export interface QualificationAssocations {
  type: "qualification";
  schemaVariantIds: string[];
  componentIds: string[];
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

export interface AttributeAssocations {
  type: "attribute";
  prototypes: AttributePrototypeView[];
  arguments: FuncArgument[];
}

export type FuncAssociations =
  | AttributeAssocations
  | CodeGenerationAssociations
  | ConfirmationAssociations
  | QualificationAssocations
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
