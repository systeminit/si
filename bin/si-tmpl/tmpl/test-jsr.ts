import { SubscriptionInput, TemplateContext } from "jsr:@systeminit/template";
import { z } from "npm:zod";

export default function (c: TemplateContext) {
  c.name("van morrison");
  c.changeSet(`${c.name()} always uses the ${c.invocationKey()}`);

  const inputSchema = z.object({
    environment: z.string().describe("the environment to build"),
    credential: SubscriptionInput,
    region: SubscriptionInput,
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
    c.setSubscription(newSubnet, "/domain/VpcId", {
      kind: "$source",
      "component": newVpc.id,
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
          c.setAttribute(copy, "/si/tags/amazingGrace", "how sweet the sound");
          c.setAttribute(copy, "/si/tags/poop", "canoyeye");
          workingSet.push(copy);
        }
      }
    }
    for (const w of workingSet) {
      if (w.attributes["/secrets/AWS Credential"]) {
        await c.setSubscription(
          w,
          "/secrets/AWS Credential",
          typedInputs.credential,
        );
      }
      if (w.attributes["/domain/extra/Region"]) {
        await c.setSubscription(w, "/domain/extra/Region", typedInputs.region);
      }
      c.setSiblingAttribute(
        w,
        /\/domain\/Tags\/\d+\/Key/,
        "Name",
        "Value",
        w.name,
      );
      c.setAttribute(w, "/si/tags/frenchFries", "are like pizza");
    }
    return workingSet.filter((comp) => comp.name.startsWith("production-"));
  });
}
