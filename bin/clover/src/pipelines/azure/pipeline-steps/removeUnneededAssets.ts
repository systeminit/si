import _logger from "../../../logger.ts";
import _ from "lodash";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
const IGNORED_CATEGORIES = [
  "Microsoft.RecoveryServices/vaults/replicationRecoveryPlans",
];
export function removeUnneededAssets(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  return specs.filter(({ schemas: [{ variants: [variant] }] }) =>
    !IGNORED_CATEGORIES.find((c) => variant.superSchema.typeName.startsWith(c))
  );
}
