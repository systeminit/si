import { ExpandedPkgSpec } from "../../spec/pkgs";
import { createDefaultPropFromCf, createDocLink, OnlyProperties } from "../../spec/props";
import { makeModule } from "../generic";
import { HetznerSchema } from "../types";
import { hCategory } from "./pipeline";
import { createDefaultProp } from "./prop";

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

  const secrets =  createDefaultPropFromCf("secrets", {}, credential, onlyProperties);

  const m = makeModule(
    credential,
    createDocLink(credential, undefined),
    credential.description,
    domain,
    resourceValue,
    secrets,
    hCategory,
  );
  
  specs.push(m);
  return specs;
}