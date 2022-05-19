import { listComponentsIdentification } from "./component/list_components_identification";
import { listQualifications } from "./component/list_qualifications";
import { getResource } from "./component/get_resource";
import { syncResource } from "./component/sync_resource";
import { checkQualifications } from "./component/check_qualifications";
import { getComponentsMetadata } from "./component/get_components_metadata";
import { getCode } from "./component/get_code";
import { generateCode } from "./component/generate_code";
import { getPropertyEditorSchema } from "./component/get_property_editor_schema";
import { getPropertyEditorValues } from "./component/get_property_editor_values";
import { getPropertyEditorValidations } from "./component/get_property_editor_validations";
import { updateFromEditField } from "./component/update_property_editor_value";
import { insertFromEditField } from "./component/insert_property_editor_value";

export const ComponentService = {
  listComponentsIdentification,
  getComponentsMetadata,
  listQualifications,
  getResource,
  syncResource,
  checkQualifications,
  getCode,
  generateCode,
  getPropertyEditorSchema,
  getPropertyEditorValues,
  getPropertyEditorValidations,
  updateFromEditField,
  insertFromEditField,
};
