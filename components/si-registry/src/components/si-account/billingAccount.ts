import { PropMethod, PropLink } from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "billingAccount",
  displayTypeName: "System Initiative Billing Account",
  siPathName: "si-account",
  serviceName: "account",
  options(c: SystemObject) {
    c.addListMethod();
    c.addGetMethod();

    // Fill in the create methods here!
    c.methods.addMethod({
      name: "create",
      label: "Create a Billing Account",
      options(p: PropMethod) {
        p.mutation = true;
        //p.isPrivate = true;
        p.skipAuth = true;
        p.request.properties.addText({
          name: "name",
          label: "Billing Account Name",
          options(p) {
            p.required = true;
          },
        });
        p.request.properties.addText({
          name: "displayName",
          label: "Billing Account Display Name",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "object",
          label: `${this.displayTypeName} Object`,
          options(p: PropLink) {
            p.lookup = {
              typeName: "billingAccount",
            };
          },
        });
      },
    });
  },
});
