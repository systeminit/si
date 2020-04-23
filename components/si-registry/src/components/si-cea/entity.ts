import { PropEnum } from "../../components/prelude";

import { registry } from "../../registry";
import { PropNumber } from "@/prop/number";

registry.base({
  typeName: "entitySiProperties",
  displayTypeName: "SI Entity Internal Properties",
  serviceName: "cea",
  options(c) {
    c.fields.addEnum({
      name: "entityState",
      label: "Entity State",
      options(p: PropEnum) {
        p.universal = true;
        p.variants = ["error", "ok", "transition"];
      },
    });
    c.fields.addText({
      name: "integrationId",
      label: "Integration Id",
      options(p) {
        p.readOnly = true;
        p.hidden = true;
        p.required = true;
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "integrationServiceId",
      label: "Integration Service Id",
      options(p) {
        p.readOnly = true;
        p.hidden = true;
        p.required = true;
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "componentId",
      label: "Component Id",
      options(p) {
        p.readOnly = true;
        p.hidden = true;
        p.required = true;
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "workspaceId",
      label: "Workspace ID",
      options(p) {
        p.readOnly = true;
        p.hidden = true;
        p.required = true;
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "organizationId",
      label: "Organization ID",
      options(p) {
        p.readOnly = true;
        p.hidden = true;
        p.required = true;
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "billingAccountId",
      label: "Billing Account ID",
      options(p) {
        p.readOnly = true;
        p.hidden = true;
        p.required = true;
        p.universal = true;
      },
    });
    c.fields.addNumber({
      name: "version",
      label: "Version",
      options(p: PropNumber) {
        p.numberKind = "int32";
        p.readOnly = true;
        p.hidden = true;
        p.required = true;
        p.universal = true;
      },
    });
  },
});
