import { ActionFuncSpecKind } from "../../bindings/ActionFuncSpecKind.ts";
import { FuncSpecInfo } from "../../spec/funcs.ts";

export const AUTHENTICATION_FUNC_SPECS = {
  "Hetzner Authentication": {
    id: "d63c1360e3b82a50d2c391b613c930fd1323dd064f0340142d962c4712e930af",
    displayName: "Authentication with Hetzner Cloud",
    path: "./src/pipelies/hetzner/funcs/authentication/authenticateHetzner.ts",
    backendKind: "jsAuthentication",
    responseType: "action",
    actionKind: "create",
  }
} as const satisfies Record<
  string,
  FuncSpecInfo & { actionKind: ActionFuncSpecKind }
>;

export const QUALIFICATION_FUNC_SPECS = {
  "Hetzner Authentication Qualification": {
    id: "f594dc6ebe7597027203a39f2bef0307f2c09d97067c1a4e1a4fb9f7f3b9d379",
    displayName: "Qualify Credentials with Hetzner Cloud",
    path: "./src/pipelies/hetzner/funcs/qualifications/credentialQualification.ts",
    backendKind: "jsAuthentication",
    responseType: "action",
    actionKind: "create",
  },
} as const as Record<
  string,
  FuncSpecInfo
>;