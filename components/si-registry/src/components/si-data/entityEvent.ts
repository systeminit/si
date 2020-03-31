import { PropEnum, PropObject, Component } from "@/components/prelude";

import { registry } from "@/componentRegistry";

registry.component({
  typeName: "entityEvent",
  displayTypeName: "SI Entity Event",
  noStd: true,
  options(c: Component) {
    c.internalOnly.addObject({
      name: "entityEventSiProperties",
      label: "Entity Event SI Properties",
      options(p: PropObject) {
        p.universal = true;
        p.properties.addText({
          name: "integrationId",
          label: "Integration Id",
          options(p) {
            p.readOnly = true;
            p.hidden = true;
            p.required = true;
            p.universal = true;
          },
        });
        p.properties.addText({
          name: "integrationServiceId",
          label: "Integration Service Id",
          options(p) {
            p.readOnly = true;
            p.hidden = true;
            p.required = true;
            p.universal = true;
          },
        });
        p.properties.addText({
          name: "componentId",
          label: "Component Id",
          options(p) {
            p.readOnly = true;
            p.hidden = true;
            p.required = true;
            p.universal = true;
          },
        });
        p.properties.addText({
          name: "workspaceId",
          label: "Workspace ID",
          options(p) {
            p.readOnly = true;
            p.hidden = true;
            p.required = true;
            p.universal = true;
          },
        });
        p.properties.addText({
          name: "organizationId",
          label: "Organization ID",
          options(p) {
            p.readOnly = true;
            p.hidden = true;
            p.required = true;
            p.universal = true;
          },
        });
        p.properties.addText({
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
  },
});
