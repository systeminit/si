import { assertEquals } from "@std/assert";
import { extractFields } from "../mod.ts";

if (Deno.env.has("OPENAI_API_KEY")) {
  Deno.test("extractFields - AutoScalingGroup fields for scaling requirements", async () => {
    const typeName = "AWS::AutoScaling::AutoScalingGroup";
    const request =
      "Can you make sure there are at least 5 and at most 50 instances, that I spend as little as possible?";
    const result = await extractFields(typeName, request);

    // Verify structure of response
    assertEquals(Array.isArray(result.properties), true);
  });

  Deno.test("extractFields - EC2 Instance fields for WordPress", async () => {
    const typeName = "AWS::EC2::Instance";
    const request =
      "can you set every field I need to launch a wordpress instance?";
    const result = await extractFields(typeName, request);

    // Verify structure of response
    assertEquals(Array.isArray(result.properties), true);
  });

  Deno.test("extractFields - handles non-AWS resource types gracefully", async () => {
    // Test with a non-AWS resource type
    const typeName = "Generic Frame";
    const request = "update the frame properties";
    const existingProperties = {
      si: {
        name: "Test Frame",
        type: "generic-frame",
      },
      domain: {
        customProp1: "value1",
        customProp2: "value2",
      },
    };

    // This should not throw an error
    const result = await extractFields(
      typeName,
      request,
      existingProperties,
    ) as {
      properties: unknown[];
    };

    // Result should have an empty properties array
    assertEquals(Array.isArray(result.properties), true);
    assertEquals(result.properties.length, 0);
  });
}
