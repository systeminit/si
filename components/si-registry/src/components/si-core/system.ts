import { PropBool, PropObject } from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "system",
  displayTypeName: "A System Initiative System",
  siPathName: "si-core",
  serviceName: "core",
  options(c: SystemObject) {
    c.associations.inList({
      fieldName: "applications",
      typeName: "applicationEntity",
      toFieldPath: ["inSystems"],
    });

    c.fields.addObject({
      name: "siProperties",
      label: "SI Internal Properties",
      options(p: PropObject) {
        p.required = true;
        p.properties.addText({
          name: "billingAccountId",
          label: "Billing Account ID",
          options(p) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "organizationId",
          label: "Organization ID",
          options(p) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "workspaceId",
          label: "Organization ID",
          options(p) {
            p.required = true;
          },
        });
      },
    });

    c.addListMethod();
    c.addGetMethod();
    c.addCreateMethod();
  },
});
