import { PropSelect } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "application",
  displayTypeName: "A System Initiative Application",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
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
});
