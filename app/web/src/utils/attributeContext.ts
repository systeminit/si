import { AttributeContext } from "../api/sdf/dal/attribute";
import { ComponentIdentification } from "../api/sdf/dal/component";
import { EditFieldBaggage } from "../api/sdf/dal/edit_field";

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
      attribute_context_internal_provider_id: -1,
      attribute_context_external_provider_id: -1,
      attribute_context_schema_id: -1,
      attribute_context_schema_variant_id: -1,
      attribute_context_component_id: -1,
      attribute_context_system_id: -1,
    };
  }

  return {
    attribute_context_prop_id: baggage.prop_id,
    attribute_context_internal_provider_id: -1,
    attribute_context_external_provider_id: -1,
    attribute_context_schema_id: componentIdentification.schemaId,
    attribute_context_schema_variant_id:
      componentIdentification.schemaVariantId,
    attribute_context_component_id: componentIdentification.componentId,
    attribute_context_system_id: -1,
  };
}
