import { PropBool } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "system",
  displayTypeName: "A System Initiative System",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.integrationServices.push({
      integrationName: "global",
      integrationServiceName: "core",
    });

    c.associations.inList({
      fieldName: "applications",
      typeName: "applicationEntity",
      toFieldPath: ["inSystems"],
    });

    // Properties
    // TODO(fnichol): we don't have properties, but GraphQL will not accept an empty
    // *Properties type so... phantom data for now?
    c.properties.addBool({
      name: "phantom",
      label: "Phantom Data",
      options(p: PropBool) {
        p.hidden = true;
        p.baseDefaultValue = true;
      },
    });
  },
});
