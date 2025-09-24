import _logger from "../../../logger.ts";
import { ExpandedPkgSpec } from "../../../spec/pkgs.ts";
import { bfsPropTree } from "../../../spec/props.ts";
import { Inferred } from "../../../spec/inferred.ts";

export function addInferredEnums(
  specs: ExpandedPkgSpec[],
  inferred: Record<string, Inferred>,
): ExpandedPkgSpec[] {
  for (const { schemas: [{ variants: [variant] }] } of specs) {
    bfsPropTree([variant.domain, variant.resourceValue], (prop) => {
      if (!prop.data.documentation) return;
      const inferredEnum = inferred[prop.data.documentation];
      if (inferredEnum?.enum) {
        if (prop.data.widgetKind !== "ComboBox") {
          prop.data.widgetKind = "ComboBox";
        }
        prop.data.widgetOptions ??= [];
        // Append any new values from the inferred enum (sometimes we're adding stuff that genuinely doesn't exist)
        for (const inferredValue of inferredEnum.enum) {
          if (
            !prop.data.widgetOptions.some(({ value }) =>
              inferredValue === value
            )
          ) {
            prop.data.widgetOptions.push({
              label: inferredValue,
              value: inferredValue,
            });
          }
        }
      }
    });
  }
  return specs;
}
