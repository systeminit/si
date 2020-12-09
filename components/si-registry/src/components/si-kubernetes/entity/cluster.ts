import { PropSelect } from "../../../components/prelude";
import { ComponentAndEntityObject } from "../../../systemComponent";
import { registry } from "../../../registry";

let kubernetesCluster = {
  typeName: "kubernetesCluster",
  displayTypeName: "Kubernetes Cluster",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: ComponentAndEntityObject) {
    c.entity.iEntity = {
      uiVisible: false,
    };
    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });

    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });
  },
};

export { kubernetesCluster };

registry.componentAndEntity(kubernetesCluster);
