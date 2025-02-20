import _logger from "../logger.ts";
import _ from "npm:lodash";
import { ExpandedPkgSpec } from "../spec/pkgs.ts";

export function removeUnneededAssets(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  return specs.filter(({ schemas: [{ variants: [variant] }] }) =>
    !variant.cfSchema.typeName.startsWith("AWS::IAM::")
  );
}
