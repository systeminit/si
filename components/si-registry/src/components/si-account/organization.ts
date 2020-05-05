import { PropObject, PropMethod, PropLink } from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "organization",
  displayTypeName: "A System Initiative Organization",
  siPathName: "si-account",
  serviceName: "account",
  options(c: SystemObject) {
    c.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
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
              typeName: "organization",
              names: ["siProperties"],
            };
          },
        });
        p.reply.properties.addLink({
          name: "item",
          label: `${c.displayTypeName} Item`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "organization",
            };
          },
        });
      },
    });
  },
});

