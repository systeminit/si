import type {
  FuncSpecData,
} from "../../../../lib/si-pkg/bindings/FuncSpecData.ts";
import { FuncSpec } from "../../../../lib/si-pkg/bindings/FuncSpec.ts";
import _ from "lodash";
import { strippedBase64 } from "../spec/funcs.ts";
import { CREATE_ONLY_PROP_LABEL, ExpandedPropSpec } from "../spec/props.ts";
import { ExpandedPkgSpec, ExpandedSchemaVariantSpec } from "../spec/pkgs.ts";

export function generateAssetFuncs(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const newSpecs = [] as ExpandedPkgSpec[];

  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;

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

function generateAssetCodeFromVariantSpec(
  variant: ExpandedSchemaVariantSpec,
): string {
  let declarations = "";
  let adds = "";

  // Code for Props
  {
    let propDeclarations = `${indent(1)}// Props\n`;
    let propAdds = "";

    for (const prop of variant.domain.entries) {
      const varName = `${prop.name}Prop`.replace(" ", "");
      propDeclarations += `${indent(1)}const ${varName} = ${
        generatePropBuilderString(prop, 2)
      };\n\n`;
      propAdds += `${indent(2)}.addProp(${varName})\n`;
    }

    declarations += propDeclarations;
    adds += propAdds;
  }

  // Code for Secret Props

  {
    if (variant.secrets.kind !== "object") {
      console.log(
        `Could not generate default props and sockets for ${variant.data?.displayName}: secrets is not object`,
      );
      throw "root/Secrets prop is not object";
    }
    let propDeclarations = `${indent(1)}// Secrets\n`;
    let propAdds = "";

    for (const prop of variant.secrets.entries) {
      const varName = `${prop.name}SecretProp`.replace(" ", "");
      propDeclarations += `${indent(1)}const ${varName} = ${
        generateSecretPropBuilderString(prop, 2)
      };\n\n`;
      propAdds += `${indent(2)}.addSecretProp(${varName})\n`;
    }
    declarations += propDeclarations;
    adds += propAdds;
  }

  declarations += "\n";

  // Code for Resource Value
  {
    let propDeclarations = `${indent(1)}// Resource\n`;
    let propAdds = "";

    for (const prop of variant.resourceValue.entries) {
      const varName = `${prop.name}Resource`.replace(" ", "");
      propDeclarations += `${indent(1)}const ${varName} = ${
        generatePropBuilderString(prop, 2)
      };\n\n`;
      propAdds += `${indent(2)}.addResourceProp(${varName})\n`;
    }

    declarations += propDeclarations;
    adds += propAdds;
  }

  // Code for Sockets
  {
    let socketDeclarations = `${indent(1)}// Sockets\n`;
    let socketAdds = "";

    // Make input sockets come before output sockets
    variant.sockets.sort((s1, s2) => {
      const comp1 = s1.data?.kind === "input" ? -1 : 1;
      const comp2 = s2.data?.kind === "input" ? -1 : 1;

      return comp1 - comp2;
    });
    for (const socket of variant.sockets) {
      const data = socket.data;
      if (!data) continue;
      // if this socket in the spec is for a secret, don't add the input socket, we'll get
      // it for free by using the SecretPropBuilder above.
      if (
        variant.secrets.entries.map((entry) => entry.name).includes(socket.name)
      ) continue;

      const varName = `${socket.name}${data.kind}Socket`.replace(" ", "");

      type AnnotationItem = {
        tokens: string[];
      };
      const annotations = JSON.parse(
        data.connectionAnnotations,
      ) as AnnotationItem[];

      socketDeclarations +=
        `${indent(1)}const ${varName} = new SocketDefinitionBuilder()\n` +
        `${indent(2)}.setName("${socket.name}")\n` +
        `${indent(2)}.setArity("${data.arity}")\n` +
        annotations.map((item: AnnotationItem) =>
          `${indent(2)}.setConnectionAnnotation("${item.tokens.join("<")}${
            ">".repeat(item.tokens.length - 1)
          }")`
        ).join("\n") + "\n" +
        `${indent(2)}.build();\n\n`;

      switch (data.kind) {
        case "input":
          socketAdds += `${indent(2)}.addInputSocket(${varName})\n`;
          break;
        case "output":
          socketAdds += `${indent(2)}.addOutputSocket(${varName})\n`;
          break;
      }
    }

    declarations += socketDeclarations;
    adds += socketAdds;
  }

  return `function main() {\n${declarations}` +
    `${indent(1)}return new AssetBuilder()\n` +
    `${adds}` +
    `${indent(2)}.build();\n` +
    `}`;
}

function generateSecretPropBuilderString(
  prop: ExpandedPropSpec,
  indent_level: number,
): string {
  return `new SecretPropBuilder()\n` +
    `${indent(indent_level)}.setName("${prop.name}")\n` +
    `${indent(indent_level)}.setSecretKind("${prop.name}")\n` +
    `${indent(indent_level)}.build()`;
}

function generatePropBuilderString(
  prop: ExpandedPropSpec,
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
      return generatePropBuilderStringInner(prop.kind, entryBlock);
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

      return generatePropBuilderStringInner("object", addChildBlock);
    }
    case "number":
      return generatePropBuilderStringInner("integer");
    case "boolean":
    case "json":
    case "string":
      return generatePropBuilderStringInner(prop.kind);
  }

  function generatePropBuilderStringInner(
    kind: string,
    inner: string = "",
  ) {
    const is_create_only = prop.metadata.createOnly ?? false;

    return `new PropBuilder()\n` +
      `${indent(indent_level)}.setName("${prop.name}")\n` +
      `${indent(indent_level)}.setKind("${kind}")\n` +
      `${indent(indent_level)}.setHidden(${prop.data?.hidden ?? false})\n` +
      generateWidgetString(
        prop.data?.widgetKind,
        is_create_only,
        indent_level,
        prop.data?.widgetOptions,
      ) +
      (
        prop.data?.defaultValue
          ? `${indent(indent_level)}.setDefaultValue(${
            JSON.stringify(prop.data.defaultValue)
          })\n`
          : ""
      ) +
      (
        prop.data?.docLink
          ? `${indent(indent_level)}.setDocLink(${
            JSON.stringify(prop.data.docLink)
          })\n`
          : ""
      ) +
      (
        prop.data?.documentation
          ? `${indent(indent_level)}.setDocumentation(${
            JSON.stringify(prop.data.documentation)
          })\n`
          : ""
      ) +
      inner +
      `${indent(indent_level)}.build()`;
  }
}

function generateWidgetString(
  widgetKind: string | undefined | null,
  create_only: boolean,
  indentLevel: number,
  options?: { label: string; value: string }[] | null,
): string {
  if (!widgetKind) {
    console.log("Unable to generate widget for prop!");
    return "";
  }

  const kind = widgetKind === "ComboBox"
    ? "comboBox"
    : widgetKind.toLowerCase();

  let widgetStr =
    `${indent(indentLevel)}.setWidget(new PropWidgetDefinitionBuilder()\n` +
    `${indent(indentLevel + 1)}.setKind("${kind}")`;

  if (create_only) {
    widgetStr += `\n${indent(indentLevel + 1)}.setCreateOnly()`;
  }

  if (options) {
    for (const option of options) {
      if (option.label === CREATE_ONLY_PROP_LABEL) continue;
      widgetStr += `\n${
        indent(indentLevel + 1)
      }.addOption("${option.label}", "${option.value}")`;
    }
  }

  widgetStr += `\n${indent(indentLevel + 1)}.build())\n`;

  return widgetStr;
}

function indent(count: number) {
  const spaces = count * 4;
  return " ".repeat(spaces);
}
