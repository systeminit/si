import { PropObject, PropMethod, PropLink } from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "workspace",
  displayTypeName: "A System Initiative Workspace",
  siPathName: "si-account",
  serviceName: "account",
  options(c: SystemObject) {
    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "organizationId"],
      typeName: "organization",
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
      },
    });
    c.addListMethod();
    c.addGetMethod();
    c.methods.addMethod({
      name: "create",
      label: "Create an Organization",
      options(p: PropMethod) {
        p.mutation = true;
        p.request.properties.addText({
          name: "name",
          label: "User Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "User Display Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addLink({
          name: "siProperties",
          label: "The SI Properties for this User",
          options(p: PropLink) {
            p.required = true;
            p.lookup = {
              typeName: "workspace",
              names: ["siProperties"],
            };
          },
        });
        p.reply.properties.addLink({
          name: "object",
          label: `${c.displayTypeName} Object`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "workspace",
            };
          },
        });
      },
    });
  },
});

