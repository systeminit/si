import { Component, PropNumber, PropObject } from "../../components/prelude";
import { registry } from "../..//componentRegistry";

registry.component({
  typeName: "component",
  displayTypeName: "SI Component",
  noStd: true,
  options(c: Component) {
    c.internalOnly.addObject({
      name: "siProperties",
      label: "Common Component SI Properties",
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
        p.properties.addNumber({
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
  },
});
