import { PropSelect } from "../../components/prelude";
import { registry } from "../../registry";

registry.componentAndEntity({
  typeName: "kubernetesCluster",
  displayTypeName: "Kubernetes Cluster",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c) {
    c.entity.associations.belongsTo({
      fromFieldPath: ["siProperties", "billingAccountId"],
      typeName: "billingAccount",
    });

    c.entity.integrationServices.push({
      integrationName: "aws",
      integrationServiceName: "eks_kubernetes",
    });

    c.properties.addSelect({
      name: "class",
      label: "Class",
      options(p: PropSelect) {
        p.options = [
          { key: "none", value: "none" },
          { key: "minikube", value: "minikube" },
          { key: "aws-eks", value: "aws-eks" },
        ];
      },
    });
  },
});
