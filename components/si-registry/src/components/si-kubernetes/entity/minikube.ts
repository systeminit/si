import { PropSelect, PropAction } from "../../../components/prelude";
import { registry } from "../../../registry";

registry.componentAndEntity({
  typeName: "minikube",
  displayTypeName: "Minikube",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c) {
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

    c.properties.addSelect({
      name: "kubernetesVersion",
      label: "Kubernetes Version",
      options(p: PropSelect) {
        p.options = [
          { key: "v1.15", value: "v1.15" },
          { key: "v1.16", value: "v1.16" },
          { key: "v1.17", value: "v1.17" },
          { key: "v1.18", value: "v1.18" },
        ];
        p.baseDefaultValue = "v1.18";
      },
    });

    c.properties.addSelect({
      name: "driver",
      label: "driver",
      options(p: PropSelect) {
        p.options = [
          { key: "none", value: "none" },
          { key: "docker", value: "docker" },
          { key: "virtualbox", value: "virtualbox" },
          { key: "podman", value: "podman" },
          { key: "vmwarefusion", value: "vmwarefusion" },
          { key: "kvm2", value: "kvm2" },
          { key: "hyperkit", value: "hyperkit" },
          { key: "hyperv", value: "hyperv" },
          { key: "vmware", value: "vmware" },
          { key: "parallels", value: "parallels" },
        ];
        p.baseDefaultValue = "docker";
      },
    });

    // Entity Actions
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
  },
});
