import { listFuncs } from "./func/list_funcs";
import { getFunc } from "./func/get_func";
import { createFunc } from "./func/create_func";
import { saveFunc } from "./func/save_func";
import { execFunc } from "./func/exec_func";
import { revertFunc } from "./func/revert_func";
import { listArguments } from "./func/list_arguments";
import { createArgument } from "./func/create_argument";

export interface QualificationAssocations {
  type: "qualification";
  schemaVariantIds: number[];
  componentIds: number[];
}

export interface AttributeAssociations {
  type: "attribute";
  props: {
    propId: number;
    name: string;
    componentId?: number;
    schemaVariantId: number;
  }[];
}

export type FuncAssociations = AttributeAssociations | QualificationAssocations;

export const FuncService = {
  listFuncs,
  getFunc,
  createFunc,
  saveFunc,
  execFunc,
  revertFunc,
  listArguments,
  createArgument,
};
