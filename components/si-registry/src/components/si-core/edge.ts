import { PropEnum, PropText, PropObject } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "edge",
  displayTypeName: "A System Initiative Edge",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.integrationServices.push({
      integrationName: "global",
      integrationServiceName: "core",
    });

    c.properties.addObject({
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

    c.properties.addObject({
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

    c.properties.addBool({
      name: "bidirectional",
      label: "Bidirectional",
    });

    c.constraints.addEnum({
      name: "edgeKind",
      label: "The kind of edge this is",
      options(p: PropEnum) {
        p.variants = ["connected"];
        p.baseDefaultValue = "connected";
      },
    });
  },
});
