import { listComponentsWithSchemaAndVariant } from "./component/list_components_with_schema_and_variant";
import { listQualifications } from "./component/list_qualifications";
import { getResource } from "./component/get_resource";
import { syncResource } from "./component/sync_resource";
import { getComponentMetadata } from "./component/get_component_metadata";
import { getCode } from "./component/get_code";

export const ComponentService = {
  listComponentsWithSchemaAndVariant,
  getComponentMetadata,
  listQualifications,
  getResource,
  syncResource,
  getCode,
};
