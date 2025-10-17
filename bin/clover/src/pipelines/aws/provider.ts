import { normalizeProperty } from "../../cfDb.ts";
import { ExpandedPropSpecFor } from "../../spec/props.ts";
import {
  PipelineOptions,
  PROVIDER_REGISTRY,
  ProviderConfig,
  ProviderFuncSpecs,
  ProviderFunctions,
  SuperSchema,
} from "../types.ts";
import { AWS_PROP_OVERRIDES, AWS_SCHEMA_OVERRIDES } from "./overrides.ts";
import {
  ACTION_FUNC_SPECS,
  CODE_GENERATION_FUNC_SPECS,
  MANAGEMENT_FUNCS,
  QUALIFICATION_FUNC_SPECS,
} from "./funcs.ts";
import { generateAwsSpecs } from "./pipeline.ts";
import { JSONSchema } from "../draft_07.ts";

function cfCategory(schema: SuperSchema): string {
  const [metaCategory, category] = schema.typeName.split("::");
  return `${metaCategory}::${category}`;
}

function createDocLink(
  { typeName }: SuperSchema,
  defName: string | undefined,
  propName?: string,
): string {
  const [topLevelRef, ...typeRefParts] = typeName.toLowerCase().split("::");
  let kebabRef = typeRefParts.join("-");

  let docLink =
    "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide";

  if (defName) {
    kebabRef += `-${defName.toLowerCase()}`;
    docLink += `/${topLevelRef}-properties-${kebabRef}.html`;
  } else {
    docLink += `/${topLevelRef}-resource-${kebabRef}.html`;
  }

  if (propName) {
    docLink += `#cfn-${kebabRef}-${propName.toLowerCase()}`;
  }
  return docLink;
}

function awsNormalizeProperty(prop: JSONSchema) {
  return normalizeProperty(prop);
}

function awsIsChildRequired(
  _schema: SuperSchema,
  parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
): boolean {
  if (parentProp?.kind === "object") {
    if (!parentProp?.metadata.required) return false;
    if (!parentProp.cfProp) return false;
    if (!("required" in parentProp.cfProp)) return false;
    return parentProp.cfProp.required?.includes(childName) ?? false;
  }
  return true;
}

async function awsLoadSchemas(options: PipelineOptions) {
  return await generateAwsSpecs(options);
}

async function awsFetchSchema() {
  const child = await new Deno.Command(Deno.execPath(), {
    args: ["run", "updateSchema"],
  }).output();

  const td = new TextDecoder();
  if (!child.success) {
    const stderr = td.decode(child.stderr).trim();
    throw new Error(`Failed to fetch AWS schema: ${stderr}`);
  }
}

const awsProviderFunctions: ProviderFunctions = {
  createDocLink,
  getCategory: cfCategory,
};

const awsProviderFuncSpecs: ProviderFuncSpecs = {
  actions: ACTION_FUNC_SPECS,
  codeGeneration: CODE_GENERATION_FUNC_SPECS,
  management: MANAGEMENT_FUNCS,
  qualification: QUALIFICATION_FUNC_SPECS,
};

export const AWS_PROVIDER_CONFIG: ProviderConfig = {
  name: "aws",
  isStable: true,
  functions: awsProviderFunctions,
  funcSpecs: awsProviderFuncSpecs,
  loadSchemas: awsLoadSchemas,
  fetchSchema: awsFetchSchema,
  metadata: {
    color: "#FF9900",
    displayName: "AWS",
    description: "Amazon Web Services CloudFormation resources",
  },
  normalizeProperty: awsNormalizeProperty,
  isChildRequired: awsIsChildRequired,
  overrides: {
    propOverrides: AWS_PROP_OVERRIDES,
    schemaOverrides: AWS_SCHEMA_OVERRIDES,
  },
};

PROVIDER_REGISTRY[AWS_PROVIDER_CONFIG.name] = AWS_PROVIDER_CONFIG;
