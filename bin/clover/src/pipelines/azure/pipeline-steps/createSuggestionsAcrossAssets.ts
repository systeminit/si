import _ from "lodash";

import { bfsPropTree } from "../../../spec/props.ts";
import pluralize from "npm:pluralize@^8.0.0";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import { addPropSuggestSource } from "../../../spec/props.ts";
import _logger from "../../../logger.ts";
const logger = _logger.ns("test").seal();

export function createSuggestionsForIds(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  const schemasByResourceTypeName = new Map<string, Set<string>>();

  // Go through all the specs and get all the things that we may possibly want to reference.
  for (const spec of specs) {
    const resourceTypeName = spec.name.split("/").pop();
    if (!resourceTypeName) continue;
    const schemas = schemasByResourceTypeName.get(resourceTypeName) ??
      new Set();
    schemas.add(spec.name);
    schemasByResourceTypeName.set(resourceTypeName, schemas);
  }

  // Iterate over all specs again and find matches for "<pluralized-schema-name>/id" props and
  // "/resource_value/id" props.
  for (const spec of specs) {
    const [schema] = spec.schemas;
    const [schemaVariant] = schema.variants;
    const domain = schemaVariant.domain;

    // TODO(nick,jkeiser): this is a bit "brute force" right now, but we need to handle suffixes
    // and prefixes. For example "pool/id" might actually need to correspond it to the
    // "Microsoft.Network/networkManagers/{networkManagerName}/ipamPools" resource. Another example
    // is "gatewayLoadBalancers" vs. "loadBalancers".
    //
    // Another issue is that this doesn't handle an array of references. We aren't stripping the
    // word "Item" from the prop name like we do for AWS assets yet.
    bfsPropTree(
      domain,
      (prop) => {
        // We will only process object props that are not "/root/domain".
        if (prop.kind !== "object" || prop.metadata.propPath.length < 3) return;

        const idProp = prop.entries.find((p) => p.name === "id");
        if (!idProp) return;
        const objectPropNamePluralized = pluralize(prop.name);

        const schemas = schemasByResourceTypeName.get(objectPropNamePluralized);
        if (!schemas) return;

        for (const schema of schemas) {
          logger.debug(
            `suggest {schema:${schema}, prop:/resource_value/id for prop ${
              idProp.metadata.propPath.join("/")
            } on ${spec.name}`,
          );
          addPropSuggestSource(idProp, {
            schema,
            prop: "/resource_value/id",
          });
        }
      },
    );
  }
  return specs;
}
