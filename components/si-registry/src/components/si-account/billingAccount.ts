import { PropMethod, PropLink } from "../../components/prelude";
import { registry } from "../../registry";

registry.system({
  typeName: "billingAccount",
  displayTypeName: "System Initiative Billing Account",
  siPathName: "si-account",
  serviceName: "account",
  options(c) {
    c.fields.addText({
      name: "shortName",
      label: "Billing Account Short Name",
      options(p) {
        p.universal = true;
        p.required = true;
      },
    });
  },
});
