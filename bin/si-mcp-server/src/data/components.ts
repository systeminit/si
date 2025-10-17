import { z } from "zod";

export const AttributesSchema = z.record(
  z.union([
    z.object({
      $source: z.union([
        z.null().describe(
          "unset the value, regardless of it is a subscription; any value can be unset in this way, and will revert to its default value. This can also be used to remove array or map elements.",
        ),
        z.object({
          "component": z.string().describe(
            "the componentName or componentId; prefer the componentId",
          ),
          "path": z.string().describe(
            "the schema path for this subscription, replacing [array] or [map] with the correct index or key, respectively",
          ),
          "func": z.string().optional().describe(
            "a function id to be used as a transformation before setting the value",
          ),
        }).describe("the component and path to source the value from"),
      ]).describe("create or unset a subscription"),
    }).describe(
      "a subscription to a source attribute on another component",
    ),
    z.string().describe("a string value"),
    z.array(z.unknown()).describe("an array of values"),
    z.boolean().describe("a boolean value"),
    z.record(z.unknown()).describe("an object value"),
    z.number().describe("a number value"),
  ]),
).describe(
  "the attributes of the component; the desired state of the resource. do not start attributes with /root",
);
