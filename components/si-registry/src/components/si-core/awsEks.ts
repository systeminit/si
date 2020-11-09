import { PropText, PropSelect } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "awsEks",
  displayTypeName: "AWS Elastic Kubernetes Service",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.inputType("kubernetesCluster");
    c.entity.inputType("awsAccessKeyCredential");

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
      name: "clusterName",
      label: "cluster name",
      options(p: PropText) {
        p.required = true;
      },
    });
  },
});
