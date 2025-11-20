import type { TemplateContext } from "../src/template/context.ts";
import type { SubscriptionInputType as _SubscriptionInputType } from "../src/template.ts";
import type { z } from "zod";

export default function (c: TemplateContext) {
  c.search(["schema:*"]);
  type Inputs = z.infer<typeof inputSchema>;
  c.transform(async (workingSet, _inputs) => {
    for (const w of workingSet) {
      await c.ensureArrayAttribute(
        w,
        "/domain/Tags",
        (e) => e.subpath === "Key",
        { Key: "Name", Value: `${w.name}` },
        { skipIfMissing: true },
      );
    }
    return workingSet.filter((comp) => comp.schemaName.startsWith("AWS::"));
  });
}
