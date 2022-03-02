import { listComponentsNamesOnly } from "./component/list_components_names_only";
import { listQualifications } from "./component/list_qualifications";
import { getResource } from "./component/get_resource";
import { syncResource } from "./component/sync_resource";
import { getComponentMetadata } from "./component/get_component_metadata";
import { getCode } from "./component/get_code";

export const ComponentService = {
  listComponentsNamesOnly,
  getComponentMetadata,
  listQualifications,
  getResource,
  syncResource,
  getCode,
};
