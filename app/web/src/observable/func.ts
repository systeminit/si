import { ReplaySubject } from "rxjs";

import { GetFuncResponse } from "@/service/func/get_func";
import { SaveFuncRequest } from "@/service/func/save_func";

export interface EditingFunc {
  modifiedFunc: GetFuncResponse;
  origFunc: GetFuncResponse;
  id: number;
}

export const saveFuncToBackend$ = new ReplaySubject<SaveFuncRequest>(1);
export const funcState$ = new ReplaySubject<EditingFunc[]>(1);
