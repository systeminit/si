import { FuncArgument } from "@/api/sdf/dal/func";
import { CodeLanguage } from "@/api/sdf/dal/code_view";
import { GetFuncResponse } from "./requests/get_func";

export type EditingFunc = GetFuncResponse;

export interface CodeGenerationAssociations {
  type: "codeGeneration";
  schemaVariantIds: number[];
  componentIds: number[];
  format: CodeLanguage;
}

export interface ConfirmationAssociations {
  type: "confirmation";
  schemaVariantIds: number[];
  componentIds: number[];
}

export interface QualificationAssocations {
  type: "qualification";
  schemaVariantIds: number[];
  componentIds: number[];
}

export interface ValidationAssociations {
  type: "validation";
  prototypes: ValidationPrototypeView[];
}

export interface ValidationPrototypeView {
  id: number;
  schemaVariantId: string;
  propId: string;
}

export interface AttributePrototypeArgumentView {
  funcArgumentId: string;
  id?: number;
  internalProviderId?: number;
}

export interface AttributePrototypeView {
  id: number;
  schemaVariantId: string;
  componentId?: number;
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
