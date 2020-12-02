import { PropAction, PropSelect } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "ec2Instance",
  displayTypeName: "ec2Instance",
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

    c.entity.properties.addSelect({
      name: "instanceType",
      label: "instanceType",
      options(p: PropSelect) {
        p.required = true;
        p.options = [
          { key: "t2.nano", value: "t2.nano" },
          { key: "t2.micro", value: "t2.micro" },
          { key: "t2.small", value: "t2.small" },
          { key: "t2.medium", value: "t2.medium" },
          { key: "t2.large", value: "t2.large" },
          { key: "t2.xlarge", value: "t2.xlarge" },
          { key: "t2.2xlarge", value: "t2.2xlarge" },
          { key: "m4.large", value: "m4.large" },
          { key: "m4.xlarge", value: "m4.xlarge" },
          { key: "m4.2xlarge", value: "m4.2xlarge" },
          { key: "m4.4xlarge", value: "m4.4xlarge" },
          { key: "m4.10xlarge", value: "m4.10xlarge" },
          { key: "m4.16xlarge", value: "m4.16xlarge" },
          { key: "m5.large", value: "m5.large" },
          { key: "m5.xlarge", value: "m5.xlarge" },
          { key: "m5.2xlarge", value: "m5.2xlarge" },
          { key: "m5.4xlarge", value: "m5.4xlarge" },
          { key: "m5.8xlarge", value: "m5.8xlarge" },
          { key: "m5.12xlarge", value: "m5.12xlarge" },
          { key: "m5.16xlarge", value: "m5.16xlarge" },
          { key: "m5.24xlarge", value: "m5.24xlarge" },
        ];
      },
    });

    // Entity Actions
    c.entity.methods.addAction({
      name: "launch",
      label: "launch",
      options(p: PropAction) {
        p.mutation = true;
      },
    });
    c.entity.methods.addAction({
      name: "start",
      label: "Start",
      options(p: PropAction) {
        p.mutation = true;
      },
    });
    c.entity.methods.addAction({
      name: "stop",
      label: "Stop",
      options(p: PropAction) {
        p.mutation = true;
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
      name: "terminate",
      label: "Terminate",
      options(p: PropAction) {
        p.mutation = true;
      },
    });
  },
});
