import { FuncArgument } from "@/api/sdf/dal/func";
import { CodeLanguage } from "@/api/sdf/dal/code_view";
import { listFuncs } from "./func/list_funcs";
import { getFunc } from "./func/get_func";
import { createFunc } from "./func/create_func";
import { saveFunc } from "./func/save_func";
import { execFunc } from "./func/exec_func";
import { revertFunc } from "./func/revert_func";
import { listInputSources } from "./func/list_input_sources";

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

export interface AttributePrototypeArgumentView {
  funcArgumentId: number;
  id?: number;
  internalProviderId?: number;
}

export interface AttributePrototypeView {
  id: number;
  schemaVariantId: number;
  componentId?: number;
  propId: number;
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
  | QualificationAssocations;

export const FuncService = {
  listFuncs,
  getFunc,
  createFunc,
  saveFunc,
  execFunc,
  revertFunc,
  listInputSources,
};
