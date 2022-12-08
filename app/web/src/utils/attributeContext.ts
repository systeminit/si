import { AttributeContext } from "../api/sdf/dal/attribute";
import { ComponentIdentification } from "../api/sdf/dal/component";
import { EditFieldBaggage } from "../api/sdf/dal/edit_field";

function nilId(): string {
  return "00000000000000000000000000";
}

export function buildAttributeContext(
  baggage: EditFieldBaggage,
  componentIdentification?: ComponentIdentification,
): AttributeContext {
  // FIXME(nick): this check is required to have the edit fields display properly, but this is
  // straight up _incorrect_. We need to require "componentIdentification" if we are going to
  // create an "attributeContext", so this code needs to be addressed sooner rather than later.
  if (!componentIdentification) {
    return {
      attribute_context_prop_id: baggage.prop_id,
      attribute_context_internal_provider_id: nilId(),
      attribute_context_external_provider_id: nilId(),
      attribute_context_component_id: nilId(),
    };
  }

  return {
    attribute_context_prop_id: baggage.prop_id,
    attribute_context_internal_provider_id: nilId(),
    attribute_context_external_provider_id: nilId(),
    attribute_context_component_id: componentIdentification.componentId,
  };
}
