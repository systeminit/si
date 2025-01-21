import { PkgSpec } from "../bindings/PkgSpec.ts";
import type {
  FuncSpecData,
} from "../../../../lib/si-pkg/bindings/FuncSpecData.ts";
import { FuncSpec } from "../../../../lib/si-pkg/bindings/FuncSpec.ts";
import { SchemaVariantSpec } from "../bindings/SchemaVariantSpec.ts";
import _ from "lodash";
import { PropSpec } from "../bindings/PropSpec.ts";
import { strippedBase64 } from "../spec/funcs.ts";

export function generateAssetFuncs(specs: PkgSpec[]): PkgSpec[] {
  const newSpecs = [] as PkgSpec[];

  for (const spec of specs) {
    const schemaVariant = spec.schemas[0]?.variants[0];

    if (!schemaVariant || !schemaVariant.data) {
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
      codeBase64: strippedBase64(assetFuncCode),
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

  // Code for Props
  let propDeclarations = `${indent(1)}// Props\n`;
  let propAdds = "";

  for (const prop of variant.domain.entries) {
    const varName = `${prop.name}Prop`;
    propDeclarations += `${indent(1)}const ${varName} = ${
      generatePropBuilderString(prop, 2)
    };\n`;
    propAdds += `${indent(2)}.addProp(${varName})\n`;
  }

  // TODO add code for sockets

  return `function main() {\n${propDeclarations}
    return new AssetBuilder()\n${propAdds}${indent(2)}.build();
}`;
}

function generatePropBuilderString(
  prop: PropSpec,
  indent_level: number,
): string {
  switch (prop.kind) {
    case "array":
    case "map": {
      const entryBlock = `${indent(indent_level)}.setEntry(\n` +
        `${indent(indent_level + 1)}${
          generatePropBuilderString(prop.typeProp, indent_level + 1)
        }\n` +
        `${indent(indent_level)})\n`;

      return `new PropBuilder()\n` +
        `${indent(indent_level)}.setKind("${prop.kind}")\n` +
        `${indent(indent_level)}.setName("${prop.name}")\n` +
        `${entryBlock}` +
        `${indent(indent_level)}.build()`;
    }
    case "object": {
      const children = prop.entries.map((p) =>
        generatePropBuilderString(p, indent_level + 1)
      );

      let addChildBlock = "";

      for (const child of children) {
        addChildBlock += `${indent(indent_level)}.addChild(\n` +
          `${indent(indent_level + 1)}${child}\n` +
          `${indent(indent_level)})\n`;
      }

      return `new PropBuilder()\n` +
        `${indent(indent_level)}.setKind("object")\n` +
        `${indent(indent_level)}.setName("${prop.name}")\n` +
        `${addChildBlock}` +
        `${indent(indent_level)}.build()`;
    }
    case "number":
      return `new PropBuilder().setName("${prop.name}").setKind("integer").build()`;
    case "boolean":
    case "json":
    case "string":
      return `new PropBuilder().setName("${prop.name}").setKind("${prop.kind}").build()`;
  }
}

function indent(count: number) {
  const spaces = count * 4;
  return " ".repeat(spaces);
}
