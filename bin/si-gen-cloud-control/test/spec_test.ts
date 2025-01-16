import {
  assert,
  assertEquals,
  assertExists,
  assertInstanceOf,
  assertThrows,
} from "@std/assert";
import { CfProperty, getPropertiesForService } from "../src/cfDb.ts";
import {
  generateSiSpecForService,
} from "../src/commands/generateSiSpecDatabase.ts";
import { PropSpec } from "../../../lib/si-pkg/bindings/PropSpec.ts";

Deno.test(function generateServiceByName() {
  // Throws if the service does not exist
  assertThrows(() => generateSiSpecForService("poop"));
  // Returns the result if it does
  const goodResult = generateSiSpecForService("AWS::EC2::VPC");
  assertEquals(goodResult.name, "AWS::EC2::VPC");

  assertInstanceOf(goodResult.schemas, Array);
  assertEquals(goodResult.schemas.length, 1);
  assertInstanceOf(goodResult.schemas[0].variants, Array);
  assertEquals(goodResult.schemas[0].variants.length, 1);

  const variant = goodResult.schemas[0].variants[0];
  assertInstanceOf(variant.domain, Object);
  validateProps(variant.domain);
});

function validateProps(prop: PropSpec) {
  switch (prop.kind) {
    case "boolean":
      assertEquals(prop.data?.widgetKind, "Checkbox");
      break;
    case "number":
    case "string":
      assertEquals(prop.data?.widgetKind, "Text");
      break;
    case "json":
      break;
    case "array":
      assertEquals(prop.data?.widgetKind, "Array");
      assertExists(prop.typeProp);
      validateProps(prop.typeProp);
      break;
    case "map":
      assertEquals(prop.data?.widgetKind, "Map");
      assertExists(prop.typeProp);
      validateProps(prop.typeProp);
      break;
    case "object":
      assertEquals(prop.data?.widgetKind, "Header");
      assertExists(prop.entries);
      Object.values(prop.entries).forEach((entry) => {
        validateProps(entry);
      });
      break;
    default:
      break;
  }
}
