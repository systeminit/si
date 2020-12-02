import { PropText, PropSelect } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "awsEks",
  displayTypeName: "AWS Elastic Kubernetes Service",
  siPathName: "si-core",
  serviceName: "core",
  options(c) {
    c.entity.iEntity = {
      uiVisible: true,
      uiMenuCategory: "aws",
      uiMenuSubCategory: "eks",
      uiMenuDisplayName: "cluster"
    };
    c.entity.inputType("kubernetesCluster");
    c.entity.inputType("aws");
    c.entity.inputType("awsAccessKeyCredential");

    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
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
