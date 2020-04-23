import { PropMethod, PropLink } from "../../components/prelude";
import { registry } from "../../registry";

registry.system({
  typeName: "billingAccount",
  displayTypeName: "System Initiative Billing Account",
  siPathName: "si-account",
  serviceName: "account",
  options(c) {
    // Fill in the create methods here!
    c.methods.addMethod({
      name: "create",
      label: "Create a Billing Account",
      options(p: PropMethod) {
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
          name: "billingAccount",
          label: "Billing Account",
          options(p: PropLink) {
            p.lookup = {
              typeName: "billingAccount",
            };
          },
        });
      },
    });

    c.methods.addMethod({
      name: "get",
      label: "Get a Billing Account",
      options(p: PropMethod) {
        p.request.properties.addText({
          name: "billingAccountId",
          label: "Billing Account ID",
          options(p) {
            p.required = true;
          },
        });
        p.reply.properties.addLink({
          name: "billingAccount",
          label: "Billing Account",
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
