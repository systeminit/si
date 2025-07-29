import { assertEquals } from "jsr:@std/assert";
import { getAttributesForService } from "./cfDb.ts";

Deno.test("getAttributesForService - basic properties", () => {
  // Test with a known AWS service that should have properties
  const attributes = getAttributesForService("AWS::EC2::Instance");

  // Should return an array
  assertEquals(Array.isArray(attributes), true);

  // Should have some attributes
  assertEquals(attributes.length > 0, true);

  // Each attribute should have required structure
  for (const attr of attributes) {
    assertEquals(typeof attr.name, "string");
    assertEquals(typeof attr.path, "string");
    assertEquals(typeof attr.required, "boolean");
    assertEquals(attr.path.startsWith("/domain"), true);
  }
});

Deno.test("getAttributesForService - invalid service", () => {
  // Test with non-existent service
  const attributes = getAttributesForService("Invalid::Service::Name");

  // Should return empty array for invalid service
  assertEquals(attributes, []);
});

Deno.test("getAttributesForService - path format", () => {
  const attributes = getAttributesForService("AWS::EC2::Instance");

  // Find some expected properties and verify path format
  const instanceTypeAttr = attributes.find((attr) =>
    attr.name === "InstanceType"
  );
  if (instanceTypeAttr) {
    assertEquals(instanceTypeAttr.path, "/domain/InstanceType");
  }

  // All paths should start with /domain
  for (const attr of attributes) {
    assertEquals(attr.path.startsWith("/domain/"), true);
  }
});

Deno.test("getAttributesForService debugging", () => {
  const attributes = getAttributesForService("AWS::EC2::Instance");
  console.log(attributes);
  assertEquals("poop", "canoe");
});
