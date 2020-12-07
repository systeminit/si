import { PropObject, PropLink } from "../../../components/prelude";

import { registry } from "../../../registry";

// Depends on:
// - kubernetesContainer

registry.base({
  typeName: "kubernetesPodSpec",
  displayTypeName: "Kubernetes Pod Spec",
  serviceName: "kubernetes",
  options(c) {
    c.fields.addObject({
      name: "imagePullSecrets",
      label: "Image Pull Secrets",
      options(p: PropObject) {
        p.repeated = true;
        p.properties.addText({
          name: "name",
          label: "name",
        });
      },
    });
    c.fields.addLink({
      name: "containers",
      label: "Containers",
      options(p: PropLink) {
        p.repeated = true;
        p.lookup = {
          typeName: "kubernetesContainer",
        };
      },
    });
  },
});
