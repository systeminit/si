import type { TemplateContext } from "../src/template/context.ts";
import type { SubscriptionInputType } from "../src/template.ts";
import { z } from "zod";

export default function (c: TemplateContext) {
  c.name("van morrison");
  c.changeSet(`${c.name()} always uses the ${c.invocationKey()}`);

  const inputSchema = z.object({
    environment: z.string().describe("the environment to build"),
    credential: z.custom<SubscriptionInputType>(),
    region: z.custom<SubscriptionInputType>(),
  });

  c.inputs(inputSchema);
  c.search([
    "schema:*",
  ]);
  c.namePattern([
    { pattern: /demo-(.+)/g, replacement: "<%= inputs.environment %>-$1" },
  ]);

  type Inputs = z.infer<typeof inputSchema>;

  c.transform(async (workingSet, inputs) => {
    const typedInputs = inputs as Inputs;
    const newVpc = await c.newComponent("AWS::EC2::VPC", "production-vpc", {
      "/domain/CidrBlock": "10.0.0.0/24",
    });
    const newSubnet = await c.newComponent(
      "AWS::EC2::Subnet",
      "production-subnet",
    );
    await c.ensureAttribute(newSubnet, "/domain/VpcId", {
      kind: "$source",
      component: newVpc.id,
      path: "/resource_value/VpcId",
    });
    workingSet.push(newVpc);
    workingSet.push(newSubnet);
    for (const w of workingSet) {
      if (w.name == "production-web-server-2") {
        for (let i = 0; i < 10; i++) {
          if (i == 2) {
            continue;
          }
          const copy = c.copyComponent(w, `production-web-server-${i}`);
          await c.ensureAttribute(
            copy,
            "/si/tags/amazingGrace",
            "how sweet the sound",
          );
          await c.ensureAttribute(copy, "/si/tags/poop", "canoyeye");
          workingSet.push(copy);
        }
      }
    }
    for (const w of workingSet) {
      // ensureAttribute automatically resolves SubscriptionInputType - no conditionals needed!
      await c.ensureAttribute(
        w,
        "/secrets/AWS Credential",
        typedInputs.credential,
      );

      await c.ensureAttribute(
        w,
        "/domain/extra/Region",
        typedInputs.region,
      );

      // Schema validation with skipIfMissing option
      // This will only set the attribute if it exists in the schema
      await c.ensureAttribute(
        w,
        "/domain/OptionalField",
        "optional-value",
        { skipIfMissing: true },
      );

      // ensureArrayAttribute also resolves SubscriptionInputType automatically
      await c.ensureArrayAttribute(
        w,
        "/domain/Tags",
        (e) => e.subpath === "Key" && e.value === "Name",
        { Value: w.name },
      );

      await c.ensureAttribute(w, "/si/tags/frenchFries", "are like pizza");
    }
    return workingSet.filter((comp) => comp.name.startsWith("production-"));
  });
}
