import { CallToolResult } from "@modelcontextprotocol/sdk/types.js";
import { errorResponse } from "../tools/commonBehavior.ts";

export function validateSchemaPrereqs(
  schemaName: string,
  attributes: Record<string, unknown>,
): CallToolResult | null {
  // quick lookups instead of looping
  const has = (key: string) =>
    Object.prototype.hasOwnProperty.call(attributes, key);

  if (schemaName.startsWith("AWS")) {
    const hasRegion = has("/domain/extra/Region");
    const hasCredential = has("/secrets/AWS Credential");

    if (!hasRegion || !hasCredential) {
      return errorResponse({
        response: { status: "bad prereq", data: {} },
        message:
          "This is an AWS resource, and to import it we must have /domain/extra/Region set to a valid value or subscription, and /secrets/AWS Credential set to a subscription.",
      });
    }
  }

  if (schemaName.startsWith("Hetzner")) {
    const hasCredential = has("/secrets/Hetzner Api Token");
    if (!hasCredential) {
      return errorResponse({
        response: { status: "bad prereq", data: {} },
        message:
          "This is a Hetzner resource, and to import it we must have /secrets/Hetzner API Token set to a subscription.",
      });
    }
  }

  if (schemaName.startsWith("Microsoft")) {
    const hasCredential = has("/secrets/Microsoft Credential");
    const hasLocation = has("/domain/location");
    const hasSubscriptionDetails = has("/domain/subscriptionId");
    if (!hasLocation || !hasCredential || !hasSubscriptionDetails) {
      return errorResponse({
        response: { status: "bad prereq", data: {} },
        message:
          "This is an Azure / Microsoft resource, and to import it we must have /domain/location and /domain/subscriptionId set to a valid value or subscription and /secrets/Microsoft Credential set to a subscription.",
      });
    }
  }

  if (schemaName.startsWith("DigitalOcean")) {
    const hasCredential = has("/secrets/DigitalOcean Credential");
    if (!hasCredential) {
      return errorResponse({
        response: { status: "bad prereq", data: {} },
        message:
          "This is a DigitalOcean resource, and to import it we must have /secrets/DigitalOcean Credential set to a subscription.",
      });
    }
  }

  return null;
}
