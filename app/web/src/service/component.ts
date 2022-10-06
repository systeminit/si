import { getComponentsMetadata } from "./component/get_components_metadata";
import { getCode } from "./component/get_code";
import { generateCode } from "./component/generate_code";
import { getPropertyEditorSchema } from "./component/get_property_editor_schema";
import { getPropertyEditorValues } from "./component/get_property_editor_values";
import { getPropertyEditorValidations } from "./component/get_property_editor_validations";
import { updateFromEditField } from "./component/update_property_editor_value";
import { insertFromEditField } from "./component/insert_property_editor_value";
import { getDiff } from "./component/get_diff";

export const ComponentService = {
  getComponentsMetadata,
  getCode,
  getDiff,
  generateCode,
  getPropertyEditorSchema,
  getPropertyEditorValues,
  getPropertyEditorValidations,
  updateFromEditField,
  insertFromEditField,
};
