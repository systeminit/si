import { listFuncs } from "./func/list_funcs";
import { getFunc } from "./func/get_func";
import { createFunc } from "./func/create_func";
import { saveFunc } from "./func/save_func";
import { execFunc } from "./func/exec_func";
import { revertFunc } from "./func/revert_func";

export const FuncService = {
  listFuncs,
  getFunc,
  createFunc,
  saveFunc,
  execFunc,
  revertFunc,
};
