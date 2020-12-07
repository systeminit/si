import { PropText, PropAction, PropSelect } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "ubuntu",
  displayTypeName: "ubuntu",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.iEntity = {
      uiVisible: false,
    };
    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });
    // You're so full of shit, man.
    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });

    c.entity.properties.addSelect({
      name: "version",
      label: "version",
      options(p: PropSelect) {
        p.options = [
          {
            key: "20.04 LTS",
            value: "20.04 LTS",
          },
          {
            key: "18.04 LTS",
            value: "18.04 LTS",
          },
          {
            key: "20.04.1 LTS",
            value: "20.04.1 LTS",
          },
          {
            key: "20.04.2 LTS",
            value: "20.04.2 LTS",
          },
          {
            key: "18.04.5 LTS",
            value: "18.04.5 LTS",
          },
          {
            key: "18.04.4 LTS",
            value: "18.04.4 LTS",
          },
          {
            key: "18.04.4 LTS",
            value: "18.04.4 LTS",
          },
          {
            key: "18.04.3 LTS",
            value: "18.04.3 LTS",
          },
          {
            key: "18.04.2 LTS",
            value: "18.04.2 LTS",
          },
          {
            key: "18.04.1 LTS",
            value: "18.04.1 LTS",
          },
        ];
      },
    });

    c.entity.methods.addAction({
      name: "reboot",
      label: "Reboot",
      options(p: PropAction) {
        p.mutation = true;
      },
    });
    c.entity.methods.addAction({
      name: "halt",
      label: "Halt",
      options(p: PropAction) {
        p.mutation = true;
      },
    });
  },
});
