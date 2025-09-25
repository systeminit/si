import _ from "lodash";
import { FuncSpec } from "../../bindings/FuncSpec.ts";
import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import {
  createDefaultPropFromCf,
  createDocLink,
  OnlyProperties,
} from "../../spec/props.ts";
import {
  createAuthenticationFuncSpec,
  createDefaultFuncSpec,
  createLeafFuncSpec,
} from "../../spec/funcs.ts";
import { makeModule } from "../generic/index.ts";
import { HetznerSchema } from "../types.ts";
import {
  AUTHENTICATION_FUNC_SPECS,
  QUALIFICATION_FUNC_SPECS,
} from "./funcs.ts";
import { hCategory } from "./pipeline.ts";
import { createDefaultProp } from "./prop.ts";

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
    credential.properties,
    onlyProperties,
    credential,
  );

  const resourceValue = createDefaultProp(
    "resource_value",
    credential.properties,
    onlyProperties,
    credential,
  );

  const secrets = createDefaultPropFromCf(
    "secrets",
    {},
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
