import { CfProperty, CfSchema } from "../../../cfDb.ts";
import {
  createDefaultPropFromCf,
  OnlyProperties,
} from "../../../spec/props.ts";
import {
  ExpandedPkgSpec,
} from "../../../spec/pkgs.ts";
import { makeModule } from "../../generic/index.ts";
import { SuperSchema } from "../../types.ts";

export function cfCategory(schema: CfSchema): string {
  const [metaCategory, category] = schema.typeName.split("::");
  return `${metaCategory}::${category}`;
}

export function createDocLink(
  { typeName }: SuperSchema,
  defName: string | undefined,
  propName?: string,
): string {
  // Figure out the snake case name of the resource to link to

  // AWS::EC2::SecurityGroup -> aws, ec2-securitygroup
  const [topLevelRef, ...typeRefParts] = typeName.toLowerCase().split("::");
  let kebabRef = typeRefParts.join("-");

  let docLink =
    "https://docs.aws.amazon.com/AWSCloudFormation/latest/UserGuide";

  // If the document refers to a definition, the link is a little different
  if (defName) {
    // AWS::EC2::SecurityGroup #/definitions/Ingress -> /aws-properties-ec2-securitygroup-ingress
    kebabRef += `-${defName.toLowerCase()}`;
    docLink += `/${topLevelRef}-properties-${kebabRef}.html`;
  } else {
    docLink += `/${topLevelRef}-resource-${kebabRef}.html`;
  }

  // If a property name is provided, reference the property with a fragment
  if (propName) {
    docLink += `#cfn-${kebabRef}-${propName.toLowerCase()}`;
  }
  return docLink;
}

export function pkgSpecFromCf(cfSchema: CfSchema): ExpandedPkgSpec {
  const [metaCategory, category, name] = cfSchema.typeName.split("::");

  if (!["AWS", "Alexa"].includes(metaCategory) || !category || !name) {
    throw `Bad typeName: ${cfSchema.typeName}`;
  }

  const onlyProperties: OnlyProperties = {
    createOnly: normalizeOnlyProperties(cfSchema.createOnlyProperties),
    readOnly: normalizeOnlyProperties(cfSchema.readOnlyProperties),
    writeOnly: normalizeOnlyProperties(cfSchema.writeOnlyProperties),
    primaryIdentifier: normalizeOnlyProperties(cfSchema.primaryIdentifier),
  };

  const domain = createDefaultPropFromCf(
    "domain",
    pruneDomainValues(cfSchema.properties, onlyProperties),
    cfSchema,
    onlyProperties,
    createDocLink,
  );

  const resourceValue = createDefaultPropFromCf(
    "resource_value",
    pruneResourceValues(cfSchema.properties, onlyProperties),
    cfSchema,
    onlyProperties,
    createDocLink,
  );

  const secrets =  createDefaultPropFromCf("secrets", {}, cfSchema, onlyProperties, createDocLink);

  return makeModule(
    cfSchema,
    createDocLink(cfSchema, undefined),
    cfSchema.description,
    domain,
    resourceValue,
    secrets,
    cfCategory,
  )
}
// Remove all read only props from this list, since readonly props go on the
// resource value tree
function pruneDomainValues(
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
): Record<string, CfProperty> {
  if (!properties || !onlyProperties.readOnly) {
    return {};
  }

  const readOnlySet = new Set(onlyProperties.readOnly);
  return Object.fromEntries(
    Object.entries(properties)
      // Include properties that either have a type OR have oneOf/anyOf
      .filter(
        ([name, prop]) =>
          (prop.type || prop.oneOf || prop.anyOf) && !readOnlySet.has(name),
      ),
  );
}

function pruneResourceValues(
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
): Record<string, CfProperty> {
  if (!properties || !onlyProperties?.readOnly) {
    return {};
  }

  const readOnlySet = new Set(onlyProperties.readOnly);
  return Object.fromEntries(
    Object.entries(properties)
      // Include properties that either have a type OR have oneOf/anyOf
      .filter(
        ([name, prop]) =>
          (prop.type || prop.oneOf || prop.anyOf) && readOnlySet.has(name),
      ),
  );
}

function normalizeOnlyProperties(props: string[] | undefined): string[] {
  const newProps: string[] = [];
  for (const prop of props ?? []) {
    const newProp = prop.split("/").pop();
    if (newProp) {
      newProps.push(newProp);
    }
  }
  return newProps;
}
