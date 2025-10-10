import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../../spec/funcs.ts";
import { CfHandlerKind } from "../types.ts";

export const ACTION_FUNC_SPECS = {
  "Refresh Asset": {
    id: "c208d6f5dbcefb3eaa652bb3ab355319141d4addbc93af5b400fe879a5725f62",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "refresh",
    displayName: "Refresh an Azure Asset",
    path: "./src/pipelines/azure/funcs/actions/refresh.ts",
  },
  "Create Asset": {
    id: "fd06d31cb364876bf41751cfac4f78bea16445fffe320e2bf2eb657be9e2415c",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "create",
    displayName: "Create an Azure Asset",
    path: "./src/pipelines/azure/funcs/actions/create.ts",
  },
  "Update Asset": {
    id: "bffa8a9f05c2c57d5ad67d9c4232ab66f00a8397244f4584b571ec1d6818a9c9",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "update",
    displayName: "Update an Azure Asset",
    path: "./src/pipelines/azure/funcs/actions/update.ts",
  },
  "Delete Asset": {
    id: "7a3b87488680e187612679a5da6c4c88959b74cee9669a75c3003208afb051d7",
    backendKind: "jsAction",
    responseType: "action",
    actionKind: "delete",
    displayName: "Delete an Azure Asset",
    path: "./src/pipelines/azure/funcs/actions/delete.ts",
  },
} as const satisfies Record<
  string,
  FuncSpecInfo & { actionKind: ActionFuncSpecKind }
>;

export const CODE_GENERATION_FUNC_SPECS = {} as const satisfies Record<
  string,
  FuncSpecInfo
>;

export const MANAGEMENT_FUNCS = {} as const satisfies Record<
  string,
  FuncSpecInfo & { handlers: CfHandlerKind[] }
>;

export const QUALIFICATION_FUNC_SPECS = {} as const satisfies Record<
  string,
  FuncSpecInfo
>;
