import { ExpandedPkgSpec } from "../../spec/pkgs.ts";
import { makeModule } from "../generic/index.ts";
import { awsProviderConfig } from "./provider.ts";
import type { CfSchema } from "./schema.ts";

export function pkgSpecFromCf(cfSchema: CfSchema): ExpandedPkgSpec {
  const [metaCategory, category, name] = cfSchema.typeName.split("::");

  if (!["AWS", "Alexa"].includes(metaCategory) || !category || !name) {
    throw `Bad typeName: ${cfSchema.typeName}`;
  }

  const onlyProperties = awsProviderConfig.classifyProperties(cfSchema);

  return makeModule(
    cfSchema,
    cfSchema.description,
    onlyProperties,
    awsProviderConfig,
  );
}
