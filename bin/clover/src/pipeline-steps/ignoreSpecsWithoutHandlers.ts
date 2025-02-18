import { ExpandedPkgSpec } from "../spec/pkgs.ts";
import _logger from "../logger.ts";

const logger = _logger.ns("assetOverrides").seal();

export function ignoreSpecsWithoutHandlers(
  specs: ExpandedPkgSpec[],
): ExpandedPkgSpec[] {
  return specs.filter(({ schemas: [{ variants: [variant] }] }) => {
    if (!variant.cfSchema.handlers) {
      logger.debug(
        `Ignoring ${variant.cfSchema.typeName} because it has no handlers`,
      );
      return false;
    }
    return true;
  });
}
