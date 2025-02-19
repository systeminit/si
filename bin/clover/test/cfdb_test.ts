import { assertEquals, assertExists, assertThrows } from "@std/assert";
import {
  CfProperty,
  getPropertiesForService,
  getServiceByName,
  loadCfDatabase,
} from "../src/cfDb.ts";
import { assertObjectMatch } from "@std/assert/object-match";
import { dirname, fromFileUrl, join } from "@std/path";

const TEST_FILES = join(dirname(fromFileUrl(import.meta.url)), "test-files");
await loadCfDatabase({ path: TEST_FILES });

Deno.test(function getByServiceNameReturnsSchema() {
  // Throws if the service does not exist
  assertThrows(() => getServiceByName("poop"));
  // Returns the result if it does
  const goodResult = getServiceByName("AWS::EC2::VPC");
  assertEquals(goodResult.typeName, "AWS::EC2::VPC");
});

Deno.test(function propertiesExpandRefs() {
  const properties = getPropertiesForService("AWS::WAF::WebACL");
  assertExists(properties, "properties not found");
  assertObjectMatch(
    properties["DefaultAction"] as CfProperty,
    {
      "type": "object",
      "additionalProperties": false,
      "properties": {
        "Type": {
          "type": "string",
        },
      },
      "required": ["Type"],
    },
  );
});
