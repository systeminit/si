import { PropLink } from "../../../components/prelude";

import { BaseObject } from "../../../systemComponent";
import { registry } from "../../../registry";

// interface IngressServiceBackend {
//   name: string;
//   port: ServiceBackendPort;
// }

// Depends on:
// - kubernetesServiceBackendPort

let kubernetesIngressServiceBackend = {
  typeName: "kubernetesIngressServiceBackend",
  displayTypeName: "Kubernetes Ingress Service Backend",
  siPathName: "si-kubernetes",
  serviceName: "kubernetes",
  options(c: BaseObject) {
    // Properties
    c.fields.addText({
      name: "name",
      label: "Name",
    });

    c.fields.addLink({
      name: "port",
      label: "Port",
      options(p: PropLink) {
        p.lookup = {
          typeName: "kubernetesServiceBackendPort",
        };
      },
    });
  },
};

export { kubernetesIngressServiceBackend };

registry.base(kubernetesIngressServiceBackend);
