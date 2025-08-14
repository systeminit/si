import { setAnnotationOnSocket } from "../spec/sockets.ts";
import { ExpandedPropSpec } from "../spec/props.ts";
import { getOrCreateInputSocketFromProp } from "../spec/sockets.ts";
import { ExpandedPkgSpecWithSockets, ExpandedSchemaVariantSpecWithSockets } from "../spec/pkgs.ts";

//
// Any prop ending with PolicyDocument is considered an IAM policy document, with these properties:
//
// 1. It has a socket with annotation policydocument, which connects to the composable
//    Policy Document component.
// 2. It has type=text, widget=TextArea, so the user can copy/paste the JSON from howtos,
//    websites, or other tools by selecting "set manually."
//
// TODO arrays of policies (such as in the IAM Group resource).
// TODO *Policy (too many props that aren't really policy documents here)
//
export function createPolicyDocumentInputSockets(
  specs: readonly ExpandedPkgSpecWithSockets[],
) {
  for (const { schemas: [schema] } of specs) {
    const { variants: [variant] } = schema;
    createPolicyDocumentInputSocketsFromProp(variant, variant.domain);
    createPolicyDocumentInputSocketsFromProp(variant, variant.resourceValue);
  }
}

function createPolicyDocumentInputSocketsFromProp(
  variant: ExpandedSchemaVariantSpecWithSockets,
  prop: ExpandedPropSpec,
) {
  const name = prop.name.toLowerCase();
  if (
    ["string", "json"].includes(prop.kind) && name.endsWith("policydocument")
  ) {
    // Create a socket connecting to policydocument
    const socket = getOrCreateInputSocketFromProp(variant, prop);
    setAnnotationOnSocket(socket, "policydocument");
    // Make certain it's a textarea, even if it was not json in the first place
    prop.data.widgetKind = "TextArea";
  }

  // If it's a nested object, see if any children have a policy document
  if (prop.kind === "object") {
    for (const childProp of prop.entries) {
      createPolicyDocumentInputSocketsFromProp(variant, childProp);
    }
  }
}
