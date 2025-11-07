import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { OnlyProperties } from "../../spec/props.ts";
import { makeModule } from "../generic/index.ts";
import { CfProperty, SuperSchema } from "../types.ts";
import { DUMMY_PROVIDER_CONFIG } from "./provider.ts";
import { databaseSchema, DummySchema, serverSchema } from "./schema.ts";
import { htmlToMarkdown } from "../../util.ts";

function splitDummyProperties(
  schema: SuperSchema | DummySchema,
  onlyProperties: OnlyProperties,
): {
  domainProperties: Record<string, CfProperty>;
  resourceValueProperties: Record<string, CfProperty>;
} {
  if (!("properties" in schema)) throw new Error("Not DummySchema");
  const readOnlySet = new Set(onlyProperties.readOnly);
  const domainProperties: Record<string, CfProperty> = {};
  const resourceValueProperties: Record<string, CfProperty> = {};

  for (const [name, prop] of Object.entries(schema.properties)) {
    if (readOnlySet.has(name)) {
      resourceValueProperties[name] = prop as CfProperty;
    } else {
      domainProperties[name] = prop as CfProperty;
    }
  }

  return { domainProperties, resourceValueProperties };
}

export function pkgSpecFromDummy(): ExpandedPkgSpec[] {
  const schemas = [serverSchema, databaseSchema];
  const specs: ExpandedPkgSpec[] = [];

  for (const schema of schemas) {
    const onlyProperties: OnlyProperties = {
      createOnly: [],
      readOnly: ["id", "ipAddress", "status"],
      writeOnly: [],
      primaryIdentifier: ["id"],
    };

    const { domainProperties, resourceValueProperties } = splitDummyProperties(
      schema,
      onlyProperties,
    );

    const module = makeModule(
      schema,
      htmlToMarkdown(schema.description) ?? schema.description,
      onlyProperties,
      DUMMY_PROVIDER_CONFIG,
      domainProperties,
      resourceValueProperties,
    );

    specs.push(module);
  }

  return specs;
}
