import { PropMethod, PropLink } from "../../components/prelude";
import { registry } from "../../registry";

registry.system({
  typeName: "billingAccount",
  displayTypeName: "System Initiative Billing Account",
  siPathName: "si-account",
  serviceName: "account",
  options(c) {
    c.fields.addText({
      name: "id",
      label: "Billing Account ID",
      options(p) {
        p.universal = true;
        p.readOnly = true;
        p.required = true;
      },
    });
    c.fields.addText({
      name: "shortName",
      label: "Billing Account Short Name",
      options(p) {
        p.universal = true;
        p.required = true;
      },
    });
    c.fields.addText({
      name: "displayName",
      label: "Billing Account Display Name",
      options(p) {
        p.universal = true;
        p.required = true;
      },
    });
    c.fields.addLink({
      name: "siStorable",
      label: "SI Storable",
      options(p: PropLink) {
        p.universal = true;
        p.hidden = true;
        p.lookup = {
          typeName: "data",
          names: ["storable"],
        };
        p.required = true;
      },
    });
    c.fields.addMethod({
      name: "create",
      label: "Create a new Billing Account",
      options(p: PropMethod) {
        p.request;
      },
    });
  },
});
