import { PkgSpec } from "../bindings/PkgSpec.ts";
import type {
  FuncSpecData,
} from "../../../../lib/si-pkg/bindings/FuncSpecData.ts";
import { FuncSpec } from "../../../../lib/si-pkg/bindings/FuncSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import _ from "lodash";

export function generateAssetFuncs(specs: PkgSpec[]): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];

  for (const spec of specs) {
    const schemaVariant = spec.schemas[0]?.variants[0];

    if (!schemaVariant) {
      console.log(
        `Could not generate assetFunc for ${spec.name}: missing schema or variant`,
      );
      continue;
    }

    const assetFuncUniqueKey = schemaVariant.data.funcUniqueId;
    const assetFuncName = spec.name;

    const assetFuncCode = generateAssetCodeFromVariantSpec(schemaVariant);

    const assetFuncData: FuncSpecData = {
      name: assetFuncName,
      displayName: null,
      description: null,
      handler: "main",
      codeBase64: btoa(
        assetFuncCode,
      ).replace(/=/g, ""),
      backendKind: "jsSchemaVariantDefinition",
      responseType: "schemaVariantDefinition",
      hidden: false,
      link: null,
    };

    const assetFunc: FuncSpec = {
      name: assetFuncName,
      uniqueId: assetFuncUniqueKey,
      data: assetFuncData,
      deleted: false,
      isFromBuiltin: true,
      arguments: [],
    };
    spec.funcs.push(assetFunc);

    newSpecs.push(spec);
  }

  return newSpecs;
}

function generateAssetCodeFromVariantSpec(variant: SchemaVariantSpec): string {
  if (variant.domain.kind !== "object") throw "Domain prop is not object";

  const queue = _.cloneDeep(variant.domain.entries);

  let propDeclarations = "";
  let propAdds = "";

  while (queue.length > 0) {
    const prop = queue.pop();
    const varName = `${prop.name}Prop`;
    switch (prop.kind) {
      case "array":
        break;
      case "map":
        break;
      case "object":
        break;
      case "boolean":
      case "json":
      case "number":
      case "string":
        propDeclarations += `
    const ${varName} = new PropBuilder()
        .setKind("${prop.kind}")
        .setName("${prop.name}")
        .build();
`;

        propAdds += `
      .addProp(${varName})`;
        break;
    }
  }

  return `function main() {${propDeclarations}
    return new AssetBuilder()${propAdds}
      .build();
}`;
}
