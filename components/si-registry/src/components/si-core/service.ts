import { PropAction, PropText } from "../../components/prelude";
import { registry } from "../../registry";
import { PropNumber } from "@/prop/number";

registry.componentAndEntity({
  typeName: "service",
  displayTypeName: "Service",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    c.entity.integrationServices.push({
      integrationName: "global",
      integrationServiceName: "core",
    });

    // Properties
    c.properties.addText({
      name: "image",
      label: "Container Image",
      options(p: PropText) {
        p.required = true;
      },
    });
    c.properties.addNumber({
      name: "port",
      label: "Container Port",
      options(p: PropNumber) {
        p.required = true;
        p.numberKind = "uint32";
      },
    });
    c.properties.addNumber({
      name: "replicas",
      label: "Replicas",
      options(p: PropNumber) {
        p.numberKind = "uint32";
        p.baseDefaultValue = "1";
      },
    });

    // Entity Actions
    c.entity.methods.addAction({
      name: "deploy",
      label: "Deploy",
      options(p: PropAction) {
        p.mutation = true;
      },
    });
  },
});
