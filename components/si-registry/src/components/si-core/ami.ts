import { PropText, PropAction, PropSelect } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "ami",
  displayTypeName: "ami",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
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
      name: "region",
      label: "region",
      options(p: PropSelect) {
        p.required = true;
        p.options = [
          { key: "US East (N. Virginia) us-east-1", value: "us-east-1" },
          { key: "US East (Ohio) us-east-2", value: "us-east-2" },
          { key: "US West (N. California) us-west-1", value: "us-west-1" },
          { key: "US West (Oregon) us-west-2", value: "us-west-2" },
        ];
      },
    });

    c.entity.properties.addText({
      name: "operatingSystem",
      label: "operatingSystem",
      options(p: PropText) {
        p.required = true;
      },
    });

    c.entity.properties.addText({
      name: "amiId",
      label: "amiId",
      options(p: PropText) {
        p.required = true;
      },
    });

    c.entity.properties.addSelect({
      name: "rootDeviceType",
      label: "rootDeviceType",
      options(p: PropSelect) {
        p.required = true;
        p.options = [
          { key: "instance-store", value: "instance-store" },
          { key: "ebs", value: "ebs" },
        ];
      },
    });

    c.entity.properties.addSelect({
      name: "virtualization",
      label: "virtualization",
      options(p: PropSelect) {
        p.required = true;
        p.options = [
          { key: "paravirtual", value: "paravirtual" },
          { key: "hvm", value: "hvm" },
        ];
      },
    });
  },
});
