import _logger from "../../../logger.ts";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import { ExpandedPropSpec } from "../../../spec/props.ts";

export function reorderProps(specs: ExpandedPkgSpec[]): ExpandedPkgSpec[] {
  for (const { schemas: [{ variants: [variant] }] } of specs) {
    reorderPropChildren(variant.domain);
    reorderPropChildren(variant.resourceValue);
  }
  return specs;
}

function reorderPropChildren(prop: ExpandedPropSpec) {
  switch (prop.kind) {
    case "array":
    case "map":
      reorderPropChildren(prop.typeProp);
      break;
    case "object":
      // Sort by:
      // - extra always comes last
      // - scalar types first (complex types like object/etc. go last)
      // - required types first
      // - then by name
      prop.entries = prop.entries.toSorted(
        (a, b) =>
          compareBools(isExtraProp(a), isExtraProp(b)) ||
          -compareBools(isScalar(a), isScalar(b)) ||
          -compareBools(a.metadata.required, b.metadata.required) ||
          a.name.localeCompare(b.name),
      );
      for (const entry of prop.entries) reorderPropChildren(entry);
      break;
    default:
      break;
  }
}

function isExtraProp(prop: ExpandedPropSpec) {
  if (prop.metadata.propPath.length !== 3) return false;
  const [root, domain, extra] = prop.metadata.propPath;
  return root === "root" && domain === "domain" && extra === "extra";
}

function isScalar(prop: ExpandedPropSpec) {
  return !["object", "array", "map"].includes(prop.kind);
}

function compareBools(a: boolean, b: boolean) {
  return Number(a) - Number(b);
}
