import { PropEnum, PropObject, Component } from "../../components/prelude";

import { registry } from "../../componentRegistry";

registry.component({
  typeName: "entity",
  displayTypeName: "SI Entity",
  noStd: true,
  options(c: Component) {
    c.internalOnly.addObject({
      name: "entitySiProperties",
      label: "Common Entity SI Properties",
      options(p: PropObject) {
        p.universal = true;
        p.properties.addEnum({
          name: "entityState",
          label: "Entity State",
          options(p: PropEnum) {
            p.universal = true;
            p.variants = ["error", "ok", "transition"];
          },
        });
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
        p.properties.addNumber({
          name: "version",
          label: "Version",
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
