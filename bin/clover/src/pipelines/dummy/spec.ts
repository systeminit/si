import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { OnlyProperties } from "../../spec/props.ts";
import { makeModule } from "../generic/index.ts";
import { dummyProviderConfig } from "./provider.ts";
import { databaseSchema, serverSchema } from "./schema.ts";

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

    const module = makeModule(
      schema,
      schema.description,
      onlyProperties,
      dummyProviderConfig,
    );

    specs.push(module);
  }

  return specs;
}

