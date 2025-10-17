import _ from "lodash";
import { FuncSpec } from "../../../bindings/FuncSpec.ts";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import {
  createDefaultPropFromJsonSchema,
  OnlyProperties,
} from "../../../spec/props.ts";
import {
  createAuthenticationFuncSpec,
  createDefaultFuncSpec,
  createLeafFuncSpec,
  FuncSpecInfo,
} from "../../../spec/funcs.ts";
import { makeModule } from "../../generic/index.ts";
import { HetznerSchema } from "../schema.ts";
import { hetznerProviderConfig } from "./../provider.ts";

export function generateCredentialModule(specs: ExpandedPkgSpec[]) {
  const credential: HetznerSchema = {
    typeName: "Hetzner::Credential::ApiToken",
    description: "A Hetzner cloud credential connection",
    properties: {
      HetznerApiToken: { type: "string" },
    },
    requiredProperties: new Set([]),
    primaryIdentifier: [],
  };

  const onlyProperties: OnlyProperties = {
    createOnly: [],
    readOnly: [],
    writeOnly: [],
    primaryIdentifier: [],
  };

  const credentialSpec = createCredentialSpec(credential, onlyProperties);
  specs.push(credentialSpec);

  return specs;
}

function createCredentialSpec(
  credential: HetznerSchema,
  onlyProperties: OnlyProperties,
): ExpandedPkgSpec {
  const spec = makeModule(
    credential,
    credential.description,
    onlyProperties,
    hetznerProviderConfig,
    credential.properties,
    {},
  );

  const [schema] = spec.schemas;
  const [schemaVariant] = schema.variants;
  const funcs = spec.funcs;
  const leafFuncs = schemaVariant.leafFunctions;
  const authFuncs = schemaVariant.authFuncs;

  // Create the secret definition manually since it needs special handling
  const secretDefinition = createDefaultPropFromJsonSchema(
    "secret_definition",
    credential.properties,
    credential,
    onlyProperties,
    hetznerProviderConfig.functions.createDocLink,
    hetznerProviderConfig,
  );

  schemaVariant.secretDefinition = secretDefinition;

  if (schemaVariant.secretDefinition.kind === "object") {
    const apiTokenProp = schemaVariant.secretDefinition.entries.find(
      (p) => p.name === "HetznerApiToken",
    );
    if (apiTokenProp) {
      apiTokenProp.data.widgetKind = "Password";
      apiTokenProp.data.widgetOptions = [
        {
          label: "secretKind",
          value: "HetznerApiToken",
        },
      ];
    }
  }

  for (const func of createQualificationFuncs(schemaVariant.domain.uniqueId!)) {
    funcs.push(func);
    leafFuncs.push(
      createLeafFuncSpec("qualification", func.uniqueId, ["domain", "code"]),
    );
  }

  for (const func of createAuthenticationFuncs()) {
    funcs.push(func);
    authFuncs.push(createAuthenticationFuncSpec("authenticate", func.uniqueId));
  }

  return spec;
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
    ]),
  );
}

export function createAuthenticationFuncs() {
  return Object.entries(AUTHENTICATION_FUNC_SPECS).map(([func, spec]) =>
    createDefaultFuncSpec(func, spec, []),
  );
}

export const AUTHENTICATION_FUNC_SPECS = {
  "Hetzner Authentication": {
    id: "d63c1360e3b82a50d2c391b613c930fd1323dd064f0340142d962c4712e930af",
    displayName: "Authentication with Hetzner Cloud",
    path: "./src/pipelines/hetzner/funcs/authentication/authenticateHetzner.ts",
    backendKind: "jsAuthentication",
    responseType: "void",
  },
} as const satisfies Record<string, FuncSpecInfo>;

export const QUALIFICATION_FUNC_SPECS = {
  "Hetzner Authentication Qualification": {
    id: "f594dc6ebe7597027203a39f2bef0307f2c09d97067c1a4e1a4fb9f7f3b9d379",
    displayName: "Qualify Credentials with Hetzner Cloud",
    path: "./src/pipelines/hetzner/funcs/qualifications/credentialQualification.ts",
    backendKind: "jsAttribute",
    responseType: "qualification",
  },
} as const as Record<string, FuncSpecInfo>;
