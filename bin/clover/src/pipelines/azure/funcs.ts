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

export const MANAGEMENT_FUNCS = {
  "Discover on Azure": {
    id: "a82d730eac534eac4ce84954a8c1a19a817553c23bdccfcc5fc33f14c21ca923",
    backendKind: "management",
    responseType: "management",
    displayName: "Discover on Azure",
    path: "./src/pipelines/azure/funcs/management/discover.ts",
    handlers: ["list", "read"],
  },
  "Import from Azure": {
    id: "61d66b00cf1db372a49903bdd9c2f864ad0da606c320604623acdd72c4df6c37",
    backendKind: "management",
    responseType: "management",
    displayName: "Import from Azure",
    path: "./src/pipelines/azure/funcs/management/import.ts",
    handlers: ["read"],
  },
} as const satisfies Record<
  string,
  FuncSpecInfo & { handlers: CfHandlerKind[] }
>;

export const QUALIFICATION_FUNC_SPECS = {} as const satisfies Record<
  string,
  FuncSpecInfo
>;
