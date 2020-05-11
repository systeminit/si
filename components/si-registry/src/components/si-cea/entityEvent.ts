import { registry } from "../../registry";

registry.base({
  typeName: "entityEventSiProperties",
  displayTypeName: "SI Entity Event",
  serviceName: "cea",
  options(c) {
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
      name: "entityId",
      label: "Entity Id",
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
  },
});
