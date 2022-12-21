import { FuncArgument } from "@/api/sdf/dal/func";
import { GetFuncResponse } from "./requests/get_func";

export type EditingFunc = GetFuncResponse;

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
  schemaVariantId: string;
  componentId?: string;
  propId: string;
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
