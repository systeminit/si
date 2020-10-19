import { PropAction, PropText } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "dockerImage",
  displayTypeName: "Docker Image",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.inputType("service");

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
        p.sync = true;
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
