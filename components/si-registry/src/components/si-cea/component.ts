import { PropNumber } from "../../components/prelude";
import { registry } from "../../registry";

// Shared SI Component Properties
registry.base({
  typeName: "componentSiProperties",
  displayTypeName: "SI Component Internal Properties",
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
