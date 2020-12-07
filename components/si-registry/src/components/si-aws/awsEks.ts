import { PropText, PropSelect } from "../../components/prelude";
import { ComponentAndEntityObject } from "../../systemComponent";
import { registry } from "../../registry";

let awsEks = {
  typeName: "awsEks",
  displayTypeName: "AWS Elastic Kubernetes Service",
  siPathName: "si-core",
  serviceName: "core",
  options(c: ComponentAndEntityObject) {
    c.entity.iEntity = {
      uiVisible: true,
      uiMenuCategory: "aws",
      uiMenuSubCategory: "eks",
      uiMenuDisplayName: "cluster",
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
};

export { awsEks };
registry.componentAndEntity(awsEks);
