import { listFuncs } from "./func/list_funcs";
import { getFunc } from "./func/get_func";
import { createFunc } from "./func/create_func";
import { saveFunc } from "./func/save_func";
import { execFunc } from "./func/exec_func";
import { revertFunc } from "./func/revert_func";
import { listArguments } from "./func/list_arguments";
import { createArgument } from "./func/create_argument";
import { saveArgument } from "./func/save_argument";
import { deleteArgument } from "./func/delete_argument";
import { listInputSources } from "./func/list_input_sources";

export interface QualificationAssocations {
  type: "qualification";
  schemaVariantIds: number[];
  componentIds: number[];
}

export type FuncAssociations = QualificationAssocations;

export const FuncService = {
  listFuncs,
  getFunc,
  createFunc,
  saveFunc,
  execFunc,
  revertFunc,
  listArguments,
  createArgument,
  deleteArgument,
  saveArgument,
  listInputSources,
};
