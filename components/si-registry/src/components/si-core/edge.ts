import { PropEnum, PropText, PropObject } from "../../components/prelude";
import { registry } from "../../registry";
import { SystemObject } from "../../systemComponent";

registry.system({
  typeName: "edge",
  displayTypeName: "A System Initiative Edge",
  siPathName: "si-core",
  serviceName: "core",
  options(c: SystemObject) {
    c.fields.addObject({
      name: "siProperties",
      label: "SI Internal Properties",
      options(p: PropObject) {
        p.required = true;
        p.properties.addText({
          name: "billingAccountId",
          label: "Billing Account ID",
          options(p) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "organizationId",
          label: "Organization ID",
          options(p) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "workspaceId",
          label: "Organization ID",
          options(p) {
            p.required = true;
          },
        });
      },
    });

    c.fields.addObject({
      name: "headVertex",
      label: "Head Vertex",
      options(p: PropObject) {
        p.required = true;
        p.properties.addText({
          name: "id",
          label: "Head Vertex ID",
          options(p: PropText) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "typeName",
          label: "Head Vertex Type Name",
          options(p: PropText) {
            p.required = true;
          },
        });
      },
    });

    c.fields.addObject({
      name: "tailVertex",
      label: "Tail Vertex",
      options(p: PropObject) {
        p.required = true;
        p.properties.addText({
          name: "id",
          label: "Tail Vertex ID",
          options(p: PropText) {
            p.required = true;
          },
        });
        p.properties.addText({
          name: "typeName",
          label: "Tail Vertex Type Name",
          options(p: PropText) {
            p.required = true;
          },
        });
      },
    });

    c.fields.addBool({
      name: "bidirectional",
      label: "Bidirectional",
    });

    c.fields.addEnum({
      name: "edgeKind",
      label: "The kind of edge this is",
      options(p: PropEnum) {
        p.variants = ["connected"];
        p.baseDefaultValue = "connected";
      },
    });

    c.addListMethod();
    c.addGetMethod();
    c.addCreateMethod();
  },
});
