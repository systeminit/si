import { PropSelect } from "../../components/prelude";
import { ComponentAndEntityObject } from "../../systemComponent";
import { registry } from "../../registry";

let application = {
  typeName: "application",
  displayTypeName: "A System Initiative Application",
  siPathName: "si-core",
  serviceName: "core",
  options(c: ComponentAndEntityObject) {
    c.entity.iEntity = {
      uiVisible: false,
    };
    c.entity.integrationServices.push({
      integrationName: "global",
      integrationServiceName: "core",
    });

    c.properties.addSelect({
      name: "inSystems",
      label: "In Systems",
      options(p: PropSelect) {
        p.optionsFromType = "system";
        p.repeated = true;
      },
    });
  },
};

export { application };
registry.componentAndEntity(application);
