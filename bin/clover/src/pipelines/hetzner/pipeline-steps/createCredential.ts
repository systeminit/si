import _ from "lodash";
import { FuncSpec } from "../../../bindings/FuncSpec.ts";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  createDefaultPropFromCf,
  createDocLink,
  OnlyProperties,
} from "../../../spec/props.ts";
import {
  createAuthenticationFuncSpec,
  createDefaultFuncSpec,
  createLeafFuncSpec,
  FuncSpecInfo,
} from "../../../spec/funcs.ts";
import { makeModule } from "../../generic/index.ts";
import { HetznerSchema } from "../../types.ts";
import { hCategory } from "./../spec.ts";
import { createDefaultProp } from "./../prop.ts";

export function generateCredentialModule(
  specs: ExpandedPkgSpec[],
) {
  const credential: HetznerSchema = {
    typeName: "Hetzner Credential",
    description: "A Hetzner cloud bearer token",
    properties: {
      "ApiToken": { type: "string" },
    },
    requiredProperties: new Set(["ApiToken"]),
    primaryIdentifier: [],
  };

  const onlyProperties: OnlyProperties = {
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier: [],
  };

  const domain = createDefaultProp(
    "domain",
    {},
    onlyProperties,
    credential,
  );

  const resourceValue = createDefaultProp(
    "resource_value",
    {},
    onlyProperties,
    credential,
  );

  const secrets = createDefaultPropFromCf(
    "secrets",
    {},
    credential,
    onlyProperties,
  );

  const secretDefinition = createDefaultPropFromCf(
    "secrets",
    credential.properties,
    credential,
    onlyProperties,
  );

  const spec = makeModule(
    credential,
    createDocLink(credential, undefined),
    credential.description,
    domain,
    resourceValue,
    secrets,
    hCategory,
  );

  const [schema] = spec.schemas;
  const [schemaVariant] = schema.variants;

  schemaVariant.secretDefinition = secretDefinition;

  const funcs = spec.funcs;
  const leafFuncs = schemaVariant.leafFunctions;
  const authFuncs = schemaVariant.authFuncs;

  for (const func of createQualificationFuncs(domain.uniqueId!)) {
    funcs.push(func);
    leafFuncs.push(
      createLeafFuncSpec("qualification", func.uniqueId, [
        "domain",
        "code",
      ]),
    );
  }

  for (const func of createAuthenticationFuncs()) {
    funcs.push(func);
    authFuncs.push(
      createAuthenticationFuncSpec("authenticate", func.uniqueId),
    );
  }

  specs.push(spec);
  return specs;
}

export function createQualificationFuncs(domain_id: string): FuncSpec[] {
  return Object.entries(QUALIFICATION_FUNC_SPECS).map(([func, spec]) =>
    createDefaultFuncSpec(func, spec, [
      {
        name: "domain",
        kind: "object",
        elementKind: null,
        uniqueId: domain_id,
        deleted: false,
      },
    ])
  );
}

export function createAuthenticationFuncs() {
  return Object.entries(AUTHENTICATION_FUNC_SPECS).map(([func, spec]) =>
    createDefaultFuncSpec(func, spec, [])
  );
}

export const AUTHENTICATION_FUNC_SPECS = {
  "Hetzner Authentication": {
    id: "d63c1360e3b82a50d2c391b613c930fd1323dd064f0340142d962c4712e930af",
    displayName: "Authentication with Hetzner Cloud",
    path: "./src/pipelines/hetzner/funcs/authentication/authenticateHetzner.ts",
    backendKind: "jsAuthentication",
    responseType: "action",
  },
} as const satisfies Record<
  string,
  FuncSpecInfo
>;

export const QUALIFICATION_FUNC_SPECS = {
  "Hetzner Authentication Qualification": {
    id: "f594dc6ebe7597027203a39f2bef0307f2c09d97067c1a4e1a4fb9f7f3b9d379",
    displayName: "Qualify Credentials with Hetzner Cloud",
    path:
      "./src/pipelines/hetzner/funcs/qualifications/credentialQualification.ts",
    backendKind: "jsAttribute",
    responseType: "qualification",
  },
} as const as Record<
  string,
  FuncSpecInfo
>;
