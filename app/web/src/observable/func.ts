import { ReplaySubject } from "rxjs";

import { GetFuncResponse } from "@/service/func/get_func";
import { SaveFuncRequest } from "@/service/func/save_func";

export type EditingFunc = GetFuncResponse;

export const saveFuncToBackend$ = new ReplaySubject<SaveFuncRequest>(1);
