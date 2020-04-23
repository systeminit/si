import { PropText } from "../../prop/text";
import { registry } from "../../registry";

registry.base({
  typeName: "capability",
  displayTypeName: "A capability for authorization",
  serviceName: "account",
  options(c) {
    c.fields.addText({
      name: "subject",
      label: "The object the capability applies to",
      options(p) {
        p.readOnly = true;
        p.required = true;
        p.universal = true;
      },
    });
    c.fields.addText({
      name: "actions",
      label: "The actions this capability allows",
      options(p: PropText) {
        p.repeated = true;
        p.readOnly = true;
        p.required = true;
        p.universal = true;
      },
    });
  },
});
