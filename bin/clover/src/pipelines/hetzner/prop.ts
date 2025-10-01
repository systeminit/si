import {
  createPropFromCf,
  DefaultPropType,
  DocFn,
  ExpandedPropSpec,
  ExpandedPropSpecFor,
  OnlyProperties,
} from "../../spec/props.ts";
import { CfProperty, HetznerSchema, HQueue, SuperSchema } from "../types.ts";

const MAX_PROP_DEPTH = 30;

export function createDefaultProp(
  name: DefaultPropType,
  properties: Record<string, CfProperty>,
  onlyProperties: OnlyProperties,
  superSchema: HetznerSchema,
  docFn: DocFn,
) {
  let rootProp: ExpandedPropSpecFor["object"] | undefined;

  const queue: HQueue = {
    superSchema,
    onlyProperties,
    primaryIdentifier: ["id"],
    queue: [
      {
        propPath: ["root", name],
        // Pretend the prop only has the specified properties (since we split it up)
        cfProp: { ...superSchema, properties },
        parentProp: undefined,
        addTo: (prop: ExpandedPropSpec) => {
          if (prop.kind !== "object") {
            throw new Error(`${name} prop is not an object`);
          }
          // Set "rootProp" before returning it
          rootProp = prop;
        },
      },
    ],
  };

  while (queue.queue.length > 0) {
    const propArgs = queue.queue.shift()!;
    if (propArgs.propPath.length > MAX_PROP_DEPTH) {
      throw new Error(
        `Prop tree loop detected: Tried creating prop more than ${MAX_PROP_DEPTH} levels deep in the prop tree: ${propArgs.propPath}`,
      );
    }

    const prop = createPropFromCf(
      propArgs,
      queue,
      docFn,
      childIsRequired,
    );
    if (!prop) continue;
    if (propArgs.addTo) propArgs.addTo(prop);
  }

  if (!rootProp) {
    throw new Error(
      `createProp for ${superSchema.typeName} did not generate a ${name} prop`,
    );
  }

  // Top level prop doesn't actually get the generated doc; we add that to the schema instead
  rootProp.data.inputs = null;
  rootProp.data.widgetOptions = null;
  rootProp.data.hidden = false;
  rootProp.data.documentation = null;

  return rootProp;
}

function childIsRequired(
  schema: SuperSchema,
  _parentProp: ExpandedPropSpecFor["object" | "array" | "map"] | undefined,
  childName: string,
) {
  // not correctly accounting for depth here, parent prop path is missing
  // probably need to embed `required` into the record of properties somehow
  if (!("requiredProperties" in schema)) {
    throw new Error("Gave me the wrong schema!");
  }
  return schema.requiredProperties.has(childName);
}
